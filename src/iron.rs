//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::net::{ToSocketAddrs, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use futures::{future, Future, BoxFuture, Stream};
use futures_cpupool::CpuPool;

use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

use hyper::{Body, Error};
use hyper::server::{NewService, Http};

use request::HttpRequest;
use response::HttpResponse;

use error::HttpResult;

use {Request, Handler};
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

impl<H: Handler> Iron<H> {
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

        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let sock = TcpListener::bind(&addr, &handle).unwrap();

        let handler = RawService{
            addr: addr,
            handler: Arc::new(self.handler),
            protocol: Protocol::http(),
            pool: CpuPool::new(self.threads),
        };

        let http = Http::new();
        let server = sock.incoming().for_each(|(sock, remote_addr)| {
            http.bind_connection(&handle, sock, remote_addr, handler.new_service().unwrap());
            future::ok(())
        });

        core.run(server).map_err(|e| e.into())
    }
}

struct RawService<H> {
    handler: Arc<H>,
    addr: SocketAddr,
    protocol: Protocol,
    pool: CpuPool,
}

impl<H: Handler> ::hyper::server::NewService for RawService<H> {
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

impl<H: Handler> ::hyper::server::Service for RawHandler<H> {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = Error;
    type Future = BoxFuture<Self::Response,Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let addr = self.addr.clone();
        let proto = self.protocol.clone();
        let handler = self.handler.clone();
        self.pool.spawn_fn(move || {
            let mut http_res = HttpResponse::<Body>::new().with_status(status::InternalServerError);

            match Request::from_http(req, addr, &proto) {
                Ok(mut req) => {
                    // Dispatch the request, write the response back to http_res
                    handler.handle(&mut req).unwrap_or_else(|e| {
                        error!("Error handling:\n{:?}\nError was: {:?}", req, e.error);
                            e.response
                    }).write_back(&mut http_res)
                },
                Err(e) => {
                    error!("Error creating request:\n    {}", e);
                    bad_request(&mut http_res)
                }
            };
            future::ok(http_res)
        }).boxed()
    }
}

fn bad_request(mut http_res: &mut HttpResponse<Body>) {
    http_res.set_status(status::BadRequest);
}

