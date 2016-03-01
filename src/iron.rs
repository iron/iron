//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::net::{ToSocketAddrs, SocketAddr};
use std::time::Duration;
#[cfg(feature = "ssl")]
use std::path::PathBuf;

pub use hyper::server::Listening;
use hyper::server::Server;
use hyper::net::{Fresh, NetworkListener, HttpListener};


#[cfg(feature = "ssl")]
use hyper::net::{HttpsListener,NetworkStream,Ssl, Openssl};

use hyper::error::Error;

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

    /// Once listening, the local address that this server is bound to.
    addr: Option<SocketAddr>,

    /// Once listening, the protocol used to serve content.
    protocol: Option<Protocol>
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

    /// Controls the timeout for writes on existing conncetions.
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

/// Create a Hyper Server for a protocol
pub trait ServerFactory<T: NetworkListener> {
    ///  Get the protocol this Server will use
    fn protocol(&self) -> Protocol;

    /// Create the hyper::server::Server
    fn create_server(&self, sock_addr: SocketAddr) -> Result<Server<T>, Error>;
}

/// Default HTTP Server Factory
pub struct HttpServerFactory;

impl ServerFactory<HttpListener> for HttpServerFactory {
    fn protocol(&self) -> Protocol {
       Protocol::Http
    }

    fn create_server(&self, sock_addr: SocketAddr) -> Result<Server, Error> {
        Server::http(sock_addr)
    }
}


#[cfg(feature="ssl")]
/// HttpsServerFactory
pub struct HttpsServerFactory<S: Ssl + Clone + Send> {
    ssl: S
}

#[cfg(feature="ssl")]
impl HttpsServerFactory<Openssl> {
    /// Create a new HttpsServerFactory from an hyper::net::Openssl context
    pub fn new(ssl: Openssl) -> HttpsServerFactory<Openssl> {
        HttpsServerFactory {
            ssl: ssl
        }
    }
}

#[cfg(feature="ssl")]
impl ServerFactory<HttpsListener<Openssl>> for HttpsServerFactory<Openssl> {

    fn protocol(&self) -> Protocol {
        Protocol::Https
    }

    fn create_server(&self, sock_addr: SocketAddr) -> Result<Server<HttpsListener<Openssl>>, Error> {
        Server::https(sock_addr, self.ssl.clone())
    }
}



/// Protocol used to serve content. Future versions of Iron may add new protocols
/// to this enum. Thus you should not exhaustively match on its variants.
#[derive(Clone)]
pub enum Protocol {
    /// Plaintext HTTP/1
    Http,
    /// HTTP/1 over SSL/TLS
    #[cfg(feature = "ssl")]
    Https
}

impl Protocol {
    /// Return the name used for this protocol in a URI's scheme part.
    pub fn name(&self) -> &'static str {
        match *self {
            Protocol::Http => "http",
            #[cfg(feature = "ssl")]
            Protocol::Https => "https"
        }
    }
}

impl<H: Handler> Iron<H> {
    /// Kick off the server process using the HTTP protocol.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    ///
    /// The thread returns a guard that will automatically join with the parent
    /// once it is dropped, blocking until this happens.
    ///
    /// Defaults to a threadpool of size `8 * num_cpus`.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addrs` yourself and pass a parsed `SocketAddr`.
    pub fn http<A: ToSocketAddrs>(self, addr: A) -> HttpResult<Listening> {
        let http = &HttpServerFactory;
        self.listen_with(addr, 8 * ::num_cpus::get(), http, None)
    }

    /// Kick off the server process using the HTTPS protocol.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    ///
    /// The thread returns a guard that will automatically join with the parent
    /// once it is dropped, blocking until this happens.
    ///
    /// Defaults to a threadpool of size `8 * num_cpus`.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addrs` yourself and pass a parsed `SocketAddr`.
    #[cfg(feature = "ssl")]
    pub fn https<A: ToSocketAddrs>(self, addr: A, certificate: PathBuf, key: PathBuf)
                                   -> HttpResult<Listening> {

        let ssl = try!(Openssl::with_cert_and_key(certificate, key));

        self.listen_with(addr, 8 * ::num_cpus::get(), &HttpsServerFactory::new(ssl), None)
    }

    /// Kick off the server process with X threads.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addrs` yourself and pass a parsed `SocketAddr`.
    pub fn listen_with<A: ToSocketAddrs, TListener: NetworkListener + Send + 'static>(mut self, addr: A, threads: usize,
                                         server_factory: &ServerFactory<TListener>,
                                         timeouts: Option<Timeouts>) -> HttpResult<Listening> {
        let sock_addr = addr.to_socket_addrs()
            .ok().and_then(|mut addrs| addrs.next()).expect("Could not parse socket address.");

        self.addr = Some(sock_addr);
        self.protocol = Some(server_factory.protocol());

        let mut server = try!(server_factory.create_server(sock_addr));
        let timeouts = timeouts.unwrap_or_default();
        server.keep_alive(timeouts.keep_alive);
        server.set_read_timeout(timeouts.read);
        server.set_write_timeout(timeouts.write);
        server.handle_threads(self, threads)
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new(handler: H) -> Iron<H> {
        Iron { handler: handler, addr: None, protocol: None }
    }
}

impl<H: Handler> ::hyper::server::Handler for Iron<H> {
    fn handle(&self, http_req: HttpRequest, mut http_res: HttpResponse<Fresh>) {
        // Set some defaults in case request handler panics.
        // This should not be necessary anymore once stdlib's catch_panic becomes stable.
        *http_res.status_mut() = status::InternalServerError;

        // Create `Request` wrapper.
        match Request::from_http(http_req, self.addr.clone().unwrap(),
                                 self.protocol.as_ref().unwrap()) {
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

