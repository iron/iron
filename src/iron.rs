//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::net::{ToSocketAddrs, SocketAddr};
use std::path::PathBuf;

pub use hyper::server::Listening;
use hyper::server::Server;
use hyper::net::Fresh;

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
    pub addr: Option<SocketAddr>,

    /// Once listening, the protocol used to serve content.
    pub protocol: Option<Protocol>
}

/// Protocol used to serve content. Future versions of Iron may add new protocols
/// to this enum. Thus you should not exhaustively match on its variants.
#[derive(Clone)]
pub enum Protocol {
    /// Plaintext HTTP/1
    Http,
    /// HTTP/1 over SSL/TLS
    Https {
        /// Path to SSL certificate file
        certificate: PathBuf,
        /// Path to SSL private key file
        key: PathBuf
    }
}

impl Protocol {
    /// Return the name used for this protocol in a URI's scheme part.
    pub fn name(&self) -> &'static str {
        match *self {
            Protocol::Http => "http",
            Protocol::Https { .. } => "https"
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
    /// Defaults to a threadpool of size `2 * num_cpus`.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addrs` yourself and pass a parsed `SocketAddr`.
    pub fn http<A: ToSocketAddrs>(self, addr: A) -> HttpResult<Listening> {
        self.listen_with(addr, 2 * ::num_cpus::get(), Protocol::Http)
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
    /// Defaults to a threadpool of size `2 * num_cpus`.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addrs` yourself and pass a parsed `SocketAddr`.
    pub fn https<A: ToSocketAddrs>(self, addr: A, certificate: PathBuf, key: PathBuf)
                                   -> HttpResult<Listening> {
        self.listen_with(addr, 2 * ::num_cpus::get(),
                         Protocol::Https { certificate: certificate, key: key })
    }

    /// Kick off the server process with X threads.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addrs` yourself and pass a parsed `SocketAddr`.
    pub fn listen_with<A: ToSocketAddrs>(mut self, addr: A, threads: usize,
                                         protocol: Protocol) -> HttpResult<Listening> {
        let sock_addr = addr.to_socket_addrs()
            .ok().and_then(|mut addrs| addrs.next()).expect("Could not parse socket address.");

        self.addr = Some(sock_addr);

        self.protocol = Some(protocol.clone());

        let server = match protocol {
            Protocol::Http => Server::http(self),
            Protocol::Https { ref certificate, ref key } =>
                Server::https(self, certificate, key)
        };

        Ok(try!(server.listen_threads(sock_addr, threads)))
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new(handler: H) -> Iron<H> {
        Iron { handler: handler, addr: None, protocol: None }
    }

    fn bad_request(&self, mut http_res: HttpResponse<Fresh>) {
        *http_res.status_mut() = status::BadRequest;

        let http_res = match http_res.start() {
            Ok(res) => res,
            // Would like this to work, but if not *shrug*
            Err(_) => return,
        };

        // We would like this to work, but can't do anything if it doesn't.
        let _ = http_res.end();
    }
}

impl<H: Handler> ::hyper::server::Handler for Iron<H> {
    fn handle(&self, http_req: HttpRequest, http_res: HttpResponse<Fresh>) {
        // Create `Request` wrapper.
        let mut req = match Request::from_http(http_req, self.addr.clone().unwrap(),
                                               self.protocol.as_ref().unwrap()) {
            Ok(req) => req,
            Err(e) => {
                error!("Error creating request:\n    {}", e);
                return self.bad_request(http_res);
            }
        };

        // Dispatch the request
        let res = self.handler.handle(&mut req);

        match res {
            // Write the response back to http_res
            Ok(res) => res.write_back(http_res),
            Err(e) => {
                error!("Error handling:\n{:?}\nError was: {:?}", req, e.error);
                e.response.write_back(http_res);
            }
        }
    }
}

