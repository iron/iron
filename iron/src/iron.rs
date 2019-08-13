//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use futures::{future, Future};
use futures_cpupool::CpuPool;

use hyper;
use hyper::service::{NewService, Service};
use hyper::Server;
use hyper::{Body, Error};

use request::HttpRequest;
use response::HttpResponse;

use {Handler, Request, StatusCode};

/// The primary entrance point to `Iron`, a `struct` to instantiate a new server.
///
/// `Iron` contains the `Handler` which takes a `Request` and produces a
/// `Response`.
pub struct Iron<H> {
    /// Iron contains a `Handler`, which it uses to create responses for client
    /// requests.
    pub handler: Arc<H>,

    /// Server timeouts.
    pub timeouts: Timeouts,

    /// Cpu pool to run synchronus requests on.
    ///
    /// Defaults to `num_cpus`.  Note that reading/writing to the client is
    /// handled asyncronusly in a single thread.
    pub pool: CpuPool,

    /// Protocol of the incoming requests
    ///
    /// This is automatically set by the `http` and `https` functions, but
    /// can be set if you are manually constructing the hyper `http` instance.
    pub protocol: Protocol,

    /// Default host address to use when none is provided
    ///
    /// When set, this provides a default host for any requests that don't
    /// provide one.  When unset, any request without a host specified
    /// will fail.
    pub local_address: Option<SocketAddr>,
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
}

impl Default for Timeouts {
    fn default() -> Self {
        Timeouts {
            keep_alive: Some(Duration::from_secs(5)),
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
            handler: Arc::new(handler),
            protocol: Protocol::http(),
            local_address: None,
            timeouts: Timeouts::default(),
            pool: CpuPool::new_num_cpus(),
        }
    }

    /// Kick off the server process using the HTTP protocol.
    ///
    /// Call this once to begin listening for requests on the server.
    pub fn http<A>(mut self, addr: A)
    where
        A: ToSocketAddrs,
    {
        let addr: SocketAddr = addr.to_socket_addrs().unwrap().next().unwrap();
        self.local_address = Some(addr);

        let server = Server::bind(&addr)
            .tcp_keepalive(self.timeouts.keep_alive)
            .serve(self)
            .map_err(|e| eprintln!("server error: {}", e));

        hyper::rt::run(server);
    }
}

impl<H: Handler> NewService for Iron<H> {
    type ReqBody = hyper::body::Body;
    type ResBody = hyper::body::Body;
    type Error = Error;
    type Service = IronHandler<H>;
    type InitError = Error;
    type Future = future::FutureResult<Self::Service, Self::InitError>;

    fn new_service(&self) -> Self::Future {
        future::ok(IronHandler {
            handler: self.handler.clone(),
            addr: self.local_address,
            protocol: self.protocol.clone(),
            pool: self.pool.clone(),
        })
    }
}

/// This is the internal struct that translates between hyper and iron.
pub struct IronHandler<H> {
    handler: Arc<H>,
    addr: Option<SocketAddr>,
    protocol: Protocol,
    pool: CpuPool,
}

impl<H: Handler> Service for IronHandler<H> {
    type ReqBody = hyper::body::Body;
    type ResBody = hyper::body::Body;
    type Error = Error;
    type Future = Box<dyn Future<Item = HttpResponse<Self::ResBody>, Error = Self::Error> + Send>;

    fn call(&mut self, req: HttpRequest<Self::ReqBody>) -> Self::Future {
        let addr = self.addr;
        let proto = self.protocol.clone();
        let handler = self.handler.clone();

        Box::new(self.pool.spawn_fn(move || {
            let mut http_res = HttpResponse::<Body>::new(Body::empty());
            *http_res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            match Request::from_http(req, addr, &proto) {
                Ok(mut req) => {
                    // Dispatch the request, write the response back to http_res
                    handler
                        .handle(&mut req)
                        .unwrap_or_else(|e| {
                            error!("Error handling:\n{:?}\nError was: {:?}", req, e.error);
                            e.response
                        }).write_back(&mut http_res, req.method)
                }
                Err(e) => {
                    error!("Error creating request:\n    {}", e);
                    bad_request(&mut http_res)
                }
            };
            future::ok(http_res)
        }))
    }
}

fn bad_request(http_res: &mut HttpResponse<Body>) {
    *http_res.status_mut() = StatusCode::BAD_REQUEST;
}
