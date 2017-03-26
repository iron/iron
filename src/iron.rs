//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::io::{Error as IoError};
use std::net::{ToSocketAddrs, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use futures::{future, Future, Stream};
use futures_cpupool::CpuPool;

use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use tokio_io::{AsyncRead, AsyncWrite};

use hyper::{Body, Error};
use hyper::server::{NewService, Http};

use request::HttpRequest;
use response::HttpResponse;

use error::HttpResult;

#[cfg(feature = "ssl")]
use tokio_tls::TlsAcceptorExt;
#[cfg(feature = "ssl")]
use futures::sync::mpsc;
#[cfg(feature = "ssl")]
use futures::Sink;
#[cfg(feature = "ssl")]
use std::io::ErrorKind;

use {Request, AsyncHandler, Handler, BoxIronFuture, Response};
use status;

/// The primary entrance point to `Iron`, a `struct` to instantiate a new server.
///
/// `Iron` contains the `Handler` which takes a `Request` and produces a
/// `Response`.
pub struct Iron<H> {
    /// Iron contains a `Handler`, which it uses to create responses for client
    /// requests.
    pub handler: H,

    /// Server timeouts.
    pub timeouts: Timeouts,

    /// The number of request handling threads.
    ///
    /// Defaults to `8 * num_cpus`.
    pub threads: usize,
}

/// A settings struct containing a set of timeouts which can be applied to a server.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Timeouts {
    /// Controls the timeout for keep alive connections.
    ///
    /// The default is `Some(Duration::from_secs(5))`.
    ///
    /// NOTE: Setting this to None will have the effect of turning off keep alive.
    pub keep_alive: Option<Duration>,

    /// Controls the timeout for reads on existing connections.
    ///
    /// The default is `Some(Duration::from_secs(30))`
    pub read: Option<Duration>,

    /// Controls the timeout for writes on existing connections.
    ///
    /// The default is `Some(Duration::from_secs(1))`
    pub write: Option<Duration>
}

impl Default for Timeouts {
    fn default() -> Self {
        Timeouts {
            keep_alive: Some(Duration::from_secs(5)),
            read: Some(Duration::from_secs(30)),
            write: Some(Duration::from_secs(1))
        }
    }
}

#[derive(Clone)]
enum _Protocol {
    Http,
    Https,
}

/// Protocol used to serve content.
#[derive(Clone)]
pub struct Protocol(_Protocol);

impl Protocol {
    /// Plaintext HTTP/1
    pub fn http() -> Protocol {
        Protocol(_Protocol::Http)
    }

    /// HTTP/1 over SSL/TLS
    pub fn https() -> Protocol {
        Protocol(_Protocol::Https)
    }

    /// Returns the name used for this protocol in a URI's scheme part.
    pub fn name(&self) -> &str {
        match self.0 {
            _Protocol::Http => "http",
            _Protocol::Https => "https",
        }
    }
}

impl<H: AsyncHandler> Iron<H> {
    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new(handler: H) -> Iron<H> {
        Iron {
            handler: handler,
            timeouts: Timeouts::default(),
            threads: 8 * ::num_cpus::get(),
        }
    }

    /// Kick off the server process using the HTTP protocol.
    ///
    /// Call this once to begin listening for requests on the server.
    pub fn http<A>(self, addr: A) -> HttpResult<()>
        where A: ToSocketAddrs
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();

        let core = Core::new().unwrap();
        let handle = core.handle();

        let sock = TcpListener::bind(&addr, &handle).unwrap().incoming();

        return self.listen(sock, addr, Protocol::http(), core, handle);
    }

    /// Kick off the server process using the HTTPS protocol.
    ///
    /// Call this once to begin listening for requests on the server.
    #[cfg(feature = "ssl")]
    pub fn https<A, Tls>(self, addr: A, tls: Tls) -> HttpResult<()>
        where A: ToSocketAddrs, Tls: TlsAcceptorExt + 'static
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();

        let core = Core::new().unwrap();
        let handle = core.handle();

        let listener_tcp = TcpListener::bind(&addr, &handle).unwrap();

        let (tx, rx) = mpsc::channel(1);

        let ssl_acceptor = listener_tcp.incoming().for_each(move |(sock, remote_addr)| {
            let tx = tx.clone();
            tls.accept_async(sock).map_err(|e| IoError::new(ErrorKind::Other, e)).and_then(move |sock| {
                future::ok((sock, remote_addr))
            }).then(|r| {
                tx.send(r).map_err(|e| IoError::new(ErrorKind::Other, e))
            }).and_then(|_| future::ok(()))
        }).then(|_| future::ok(()));
        handle.spawn(ssl_acceptor);

        let listener = rx.then(|r| match r {
            Ok(real_r) => real_r,
            Err(e) => panic!(e),
        });

        return self.listen(listener, addr, Protocol::https(), core, handle);
    }

    /// Kick off a server process on an arbitrary `Listener`.
    ///
    /// Most use cases may call `http` and `https` methods instead of this.
    pub fn listen<L, S>(self, listener: L, addr: SocketAddr, protocol: Protocol, mut core: Core, handle: Handle) -> HttpResult<()>
        where L: Stream<Item=(S, SocketAddr), Error=IoError>,
        S: AsyncRead + AsyncWrite + 'static,
    {
        let handler = RawService{
            addr: addr,
            handler: Arc::new(self.handler),
            protocol: protocol,
            pool: CpuPool::new(self.threads),
        };

        let http = Http::new();
        let server = listener.for_each(|(sock, remote_addr)| {
            http.bind_connection(&handle, sock, remote_addr, handler.new_service().unwrap());
            future::ok(())
        });

        core.run(server).map_err(|e| e.into())
    }
}

impl<H: Handler> Iron<H> {
    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new_sync(handler: H) -> Iron<Arc<H>> {
        Iron::new(Arc::new(handler))
    }
}

struct RawService<H> {
    handler: Arc<H>,
    addr: SocketAddr,
    protocol: Protocol,
    pool: CpuPool,
}

impl<H: AsyncHandler> ::hyper::server::NewService for RawService<H> {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = ::hyper::Error;
    type Instance = RawHandler<H>;

    fn new_service(&self) -> Result<Self::Instance, ::std::io::Error> {
        Ok(RawHandler{
            handler: self.handler.clone(),
            addr: self.addr.clone(),
            protocol: self.protocol.clone(),
            pool: self.pool.clone(),
        })
    }
}

struct RawHandler<H> {
    handler: Arc<H>,
    addr: SocketAddr,
    protocol: Protocol,
    pool: CpuPool,
}

impl<H: AsyncHandler> ::hyper::server::Service for RawHandler<H> {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = Error;
    type Future = Box<Future<Item=Self::Response,Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let addr = self.addr.clone();
        let proto = self.protocol.clone();
        let handler = self.handler.clone();

        Box::new(match Request::from_http(req, addr, &proto) {
            Ok(mut req) => {
                req.extensions.insert::<CpuPoolKey>(self.pool.clone());
                Box::new(handler.async_handle(req).and_then(|(_, resp)| future::ok(resp)).or_else(move |e| {
                    error!("Error handling:\n{:?}\nError was: {:?}", e.request, e.error);
                    future::ok(e.response)
                })) as Box<Future<Item=Response,Error=Self::Error>>
            },
            Err(e) => {
                error!("Error creating request:\n    {}", e);
                Box::new(future::ok(Response::with((status::BadRequest))))
            },
        }.and_then(|resp| {
            let mut http_res = HttpResponse::<Body>::new().with_status(status::InternalServerError);
            resp.write_back(&mut http_res);
            Box::new(future::ok(http_res))
        }))
    }
}

impl<T: Handler> AsyncHandler for Arc<T> {
    fn async_handle(&self, mut req: Request) -> BoxIronFuture<(Request, Response)> {
        let me = self.clone();
        Box::new(req.extensions.get::<CpuPoolKey>().unwrap().clone().spawn_fn(move || {
            match me.handle(&mut req) {
                Ok(x) => future::ok((req, x)),
                Err(x) => future::err(x),
            }
        }))
    }
}

struct CpuPoolKey;

impl ::tmap::Key for CpuPoolKey { type Value = CpuPool; }
