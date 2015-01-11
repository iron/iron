//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::io::net::ip::{ToSocketAddr, SocketAddr};
use std::io::{Listener};

pub use hyper::server::Listening;
use hyper::server::Server;
use hyper::net::Fresh;

use request::HttpRequest;
use response::HttpResponse;

use {Request, Handler, IronResult};
use status;

/// The primary entrance point to `Iron`, a `struct` to instantiate a new server.
///
/// `Iron` contains the `Handler` which takes a `Request` and produces a
/// `Response`.
pub struct Iron<H> {
    /// Iron contains a `Handler`, which it uses to create responses for client
    /// requests.
    pub handler: H,
}

impl<H: Handler> Iron<H> {
    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    ///
    /// Defaults to a threadpool of size 100.
    pub fn listen<A: ToSocketAddr>(self, addr: A) -> IronResult<Listening> {
        let SocketAddr { ip, port } = try!(addr.to_socket_addr());

        Ok(try!(Server::http(ip, port).listen(self)))
    }

    /// Kick off the server process with X threads.
    pub fn listen_with<A: ToSocketAddr>(self, addr: A, threads: usize) -> IronResult<Listening> {
        let SocketAddr { ip, port } = try!(addr.to_socket_addr());

        Ok(try!(Server::http(ip, port).listen_threads(self, threads)))
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new(handler: H) -> Iron<H> {
        Iron { handler: handler }
    }
}

impl<H: Handler> ::hyper::server::Handler for Iron<H> {
    fn handle(&self, http_req: HttpRequest, mut http_res: HttpResponse<Fresh>) {
        // Create `Request` wrapper.
        let mut req = match Request::from_http(http_req) {
            Ok(req) => req,
            Err(e) => {
                error!("Error creating request:\n    {}", e);

                *http_res.status_mut() = status::BadRequest;

                let http_res = match http_res.start() {
                    Ok(res) => res,
                    Err(_) => return,
                };

                // We would like this to work, but can't do anything if it doesn't.
                let _ = http_res.end();
                return;
            }
        };

        // Dispatch the request
        let res = self.handler.call(&mut req).map_err(|e| {
            self.handler.catch(&mut req, e)
        });

        match res {
            // Write the response back to http_res
            Ok(res) => res.write_back(http_res),
            Err(e) => {
                // There is no Response, so create one.
                error!("Error handling:\n{}\nError was: {}", req, e);
                *http_res.status_mut() = status::BadRequest;

                let http_res = match http_res.start() {
                    Ok(res) => res,
                    Err(_) => return,
                };

                // We would like this to work, but can't do anything if it doesn't.
                let _ = http_res.end();
            }
        }
    }
}

