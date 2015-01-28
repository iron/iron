//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::old_io::net::ip::{ToSocketAddr, SocketAddr};
use std::os;

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
    pub addr: Option<SocketAddr>
}

impl<H: Handler> Iron<H> {
    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    ///
    /// Defaults to a threadpool of size `2 * num_cpus`.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addr` yourself and pass a parsed `SocketAddr`.
    pub fn listen<A: ToSocketAddr>(self, addr: A) -> HttpResult<Listening> {
        self.listen_with(addr, 2 * os::num_cpus())
    }

    /// Kick off the server process with X threads.
    ///
    /// ## Panics
    ///
    /// Panics if the provided address does not parse. To avoid this
    /// call `to_socket_addr` yourself and pass a parsed `SocketAddr`.
    pub fn listen_with<A: ToSocketAddr>(mut self, addr: A, threads: usize) -> HttpResult<Listening> {
        let sock_addr = addr.to_socket_addr()
            .ok().expect("Could not parse socket address.");
        let SocketAddr { ip, port } = sock_addr.clone();
        self.addr = Some(sock_addr);

        Ok(try!(Server::http(ip, port).listen_threads(self, threads)))
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new(handler: H) -> Iron<H> {
        Iron { handler: handler, addr: None }
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
        let mut req = match Request::from_http(http_req, self.addr.clone().unwrap()) {
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

