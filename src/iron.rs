//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::net::{ToSocketAddrs, SocketAddr};
use std::time::Duration;

pub use hyper::server::Listening;
use hyper::server::Server;
use hyper::net::{Fresh, SslServer, HttpListener, HttpsListener, NetworkListener};

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
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    ///
    /// The thread returns a guard that will automatically join with the parent
    /// once it is dropped, blocking until this happens.
    pub fn http<A>(self, addr: A) -> HttpResult<Listening>
        where A: ToSocketAddrs
    {
        HttpListener::new(addr).and_then(|l| self.listen(l, Protocol::http()))
    }

    /// Kick off the server process using the HTTPS protocol.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    ///
    /// The thread returns a guard that will automatically join with the parent
    /// once it is dropped, blocking until this happens.
    pub fn https<A, S>(self, addr: A, ssl: S) -> HttpResult<Listening>
        where A: ToSocketAddrs,
              S: 'static + SslServer + Send + Clone
    {
        HttpsListener::new(addr, ssl).and_then(|l| self.listen(l, Protocol::http()))
    }

    /// Kick off a server process on an arbitrary `Listener`.
    ///
    /// Most use cases may call `http` and `https` methods instead of this.
    pub fn listen<L>(self, mut listener: L, protocol: Protocol) -> HttpResult<Listening>
        where L: 'static + NetworkListener + Send
    {
        let handler = RawHandler {
            handler: self.handler,
            addr: try!(listener.local_addr()),
            protocol: protocol,
        };

        let mut server = Server::new(listener);
        server.keep_alive(self.timeouts.keep_alive);
        server.set_read_timeout(self.timeouts.read);
        server.set_write_timeout(self.timeouts.write);
        server.handle_threads(handler, self.threads)
    }
}

struct RawHandler<H> {
    handler: H,
    addr: SocketAddr,
    protocol: Protocol,
}

impl<H: Handler> ::hyper::server::Handler for RawHandler<H> {
    fn handle(&self, http_req: HttpRequest, mut http_res: HttpResponse<Fresh>) {
        // Set some defaults in case request handler panics.
        // This should not be necessary anymore once stdlib's catch_panic becomes stable.
        *http_res.status_mut() = status::InternalServerError;

        // Create `Request` wrapper.
        match Request::from_http(http_req, self.addr.clone(), &self.protocol) {
            Ok(mut req) => {
                // Dispatch the request, write the response back to http_res
                self.handler.handle(&mut req).unwrap_or_else(|e| {
                    error!("Error handling:\n{:?}\nError was: {:?}", req, e.error);
                    e.response
                }).write_back(http_res)
            },
            Err(e) => {
                error!("Error creating request:\n    {}", e);
                bad_request(http_res)
            }
        }
    }
}

fn bad_request(mut http_res: HttpResponse<Fresh>) {
    *http_res.status_mut() = status::BadRequest;

    // Consume and flush the response.
    // We would like this to work, but can't do anything if it doesn't.
    if let Ok(res) = http_res.start()
    {
        let _ = res.end();
    }
}

