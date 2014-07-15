//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::io::net::ip::{SocketAddr, IpAddr};

use http = http::server;
use super::chain::Chain;
use super::chain::stackchain::StackChain;

use super::response::Response;
use super::request::Request;

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
    ip: Option<IpAddr>,
    port: Option<u16>
}

impl<C: Clone> Clone for Iron<C> {
    fn clone(&self) -> Iron<C> {
        Iron {
            chain: self.chain.clone(),
            ip: self.ip.clone(),
            port: self.port
        }
    }
}

impl<C: Chain> Iron<C> {
    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This is a blocking operation, and is the final op that should be called
    /// on the `Iron` instance. Once `listen` is called, requests will be
    /// handled as defined through the `Iron's` `chain's` `Middleware`.
    pub fn listen(mut self, ip: IpAddr, port: u16) {
        use http::server::Server;
        self.ip = Some(ip);
        self.port = Some(port);
        self.serve_forever();
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server.
    ///
    /// Custom chains can be used by explicitly specifying the type, as
    /// `Iron::<CustomChain>::new()`.
    #[inline]
    pub fn new() -> Iron<C> {
        Iron {
            chain: Chain::new(),
            ip: None,
            port: None
        }
    }
}

/// Unused, but required for internal functionality.
///
/// This `impl` allows `Iron` to be used as a `Server` by
/// [rust-http](https://github.com/chris-morgan/rust-http).
/// This is not used by users of this library.
impl<C: Chain> http::Server for Iron<C> {
    fn get_config(&self) -> http::Config {
        http::Config { bind_address: SocketAddr {
            ip: self.ip.unwrap(),
            port: self.port.unwrap()
        } }
    }

    fn handle_request(&self, mut req: Request, res: &mut Response) {
        let mut chain = self.chain.clone();
        let _ = chain.dispatch(&mut req, res, None);
    }
}
