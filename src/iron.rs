//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::io::net::ip::{SocketAddr, IpAddr};
use std::sync::Arc;

use http::server as http;
use {Request, Handler};
use status;

use response::HttpResponse;
use request::HttpRequest;

/// The primary entrance point to `Iron`, a `struct` to instantiate a new server.
///
/// `Iron` contains the `Handler` which takes a `Request` and produces a
/// `Response`.
pub struct Iron<H> {
    /// Iron contains a `Handler`, which it uses to create responses for client
    /// requests.
    pub handler: H,
}

// The struct which actually listens and serves requests.
struct IronListener<H> {
    handler: Arc<H>,
    ip: IpAddr,
    port: u16
}

impl<H: Send + Sync> Clone for IronListener<H> {
    fn clone(&self) -> IronListener<H> {
        IronListener {
            // Just increment the Arc's reference count.
            handler: self.handler.clone(),
            ip: self.ip.clone(),
            port: self.port.clone()
        }
    }
}

impl<H: Handler> Iron<H> {
    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This consumes the Iron instance, but does the listening on
    /// another task, so is not blocking.
    pub fn listen(self, ip: IpAddr, port: u16) {
        use http::server::Server;

        spawn(proc() {
            IronListener {
                handler: Arc::new(self.handler),
                ip: ip,
                port: port
            }.serve_forever();
        });
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server, using the
    /// passed in `Handler`.
    pub fn new(handler: H) -> Iron<H> {
        Iron { handler: handler }
    }
}

impl<H: Handler> http::Server for IronListener<H> {
    fn get_config(&self) -> http::Config {
        http::Config {
            bind_address: SocketAddr {
                ip: self.ip,
                port: self.port
            }
        }
    }

    fn handle_request(&self, http_req: HttpRequest, http_res: &mut HttpResponse) {
        // Create `Request` wrapper.
        let mut req = match Request::from_http(http_req) {
            Ok(req) => req,
            Err(e) => {
                error!("Error getting request: {}", e);
                http_res.status = status::InternalServerError;
                let _ = http_res.write(b"Internal Server Error");
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
                http_res.status = status::InternalServerError;
                let _ = http_res.write(b"Internal Server Error");
            }
        }
    }
}
