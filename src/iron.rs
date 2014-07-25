//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::io::net::ip::{SocketAddr, IpAddr};
use std::cell::RefCell;

use http = http::server;
use super::chain::Chain;
use super::chain::stackchain::StackChain;

use super::response::{HttpResponse, Response};
use super::request::{HttpRequest, Request};

/// The "default server", using a `StackChain`.
pub type Server = Iron<StackChain>;

/// The primary entrance point to `Iron`, a `struct` to instantiate a new server.
///
/// The server can be made with a specific `Chain` (using `from_chain`)
/// or with a new `Chain` (using `new`). `Iron` is used to manage the server
/// processes:
/// `Iron.chain.link` is used to add new `Middleware`, and
/// `Iron.listen` is used to kick off a server process.
///
/// `Iron` contains the `Chain` which holds the `Middleware` necessary to run a server.
/// `Iron` is the main interface to adding `Middleware`, and has `Chain` as a
/// public field (for the sake of extensibility).
pub struct Iron<C> {
    /// Add `Middleware` to the `Iron's` `chain` so that requests
    /// are passed through those `Middleware`.
    /// `Middleware` is added to the chain with with `chain.link`.
    pub chain: C,
}

// The struct which actually listens and serves requests.
//
// IronListener holds its chain behind a RefCell to avoid a
// second clone in the implementation of .serve_forever().
#[deriving(Clone)]
struct IronListener<C> {
    chain: RefCell<C>,
    ip: IpAddr,
    port: u16
}

impl<C: Chain> Iron<C> {
    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This is a blocking operation, and is the final op that should be called
    /// on the `Iron` instance. Once `listen` is called, requests will be
    /// handled as defined through the `Iron's` `chain's` `Middleware`.
    pub fn listen(self, ip: IpAddr, port: u16) {
        use http::server::Server;

        IronListener {
            chain: RefCell::new(self.chain),
            ip: ip,
            port: port
        }.serve_forever();
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server.
    ///
    /// Custom chains can be used by explicitly specifying the type as
    /// in: `let customServer: Iron<CustomChain> = Iron::new();`
    #[inline]
    pub fn new() -> Iron<C> {
        Iron {
            chain: Chain::new(),
        }
    }
}

impl<C: Chain> http::Server for IronListener<C> {
    fn get_config(&self) -> http::Config {
        http::Config {
            bind_address: SocketAddr {
                ip: self.ip,
                port: self.port
            }
        }
    }

    fn handle_request(&self, http_req: HttpRequest, http_res: &mut HttpResponse) {
        // Create wrapper Request and Response
        let mut req = Request::from_http(http_req).unwrap();
        let mut res = Response::from_http(http_res);

        // Dispatch the request
        let _ = self.chain.borrow_mut().dispatch(&mut req, &mut res, None);

        // Write the response back to http_res
        res.write_back();
    }
}
