//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::io::net::ip::{SocketAddr, IpAddr};

use http::server::{Server, Config};
use http::server::request::{Star, AbsoluteUri, AbsolutePath, Authority};

use super::middleware::Middleware;

use super::furnace::Furnace;
use super::furnace::stackfurnace::StackFurnace;

use super::response::Response;
use super::request::Request;

pub type ServerT = Iron<StackFurnace>;

/// The primary entrance point to `Iron`, a `struct` to instantiate a new server.
///
/// The server can be made with a specific `Furnace` (using `from_furnace`)
/// or with a new `Furnace` (using `new`). `Iron` is used to manage the server
/// processes:
/// `Iron.smelt` is used to add new `Middleware`, and
/// `Iron.listen` is used to kick off a server process.
///
/// `Iron` contains the `Furnace` which holds the `Middleware` necessary to run a server.
/// `Iron` is the main interface to adding `Middleware`, and has `Furnace` as a
/// public field (for the sake of extensibility).
pub struct Iron<F> {
    /// This is exposed for the sake of extensibility. It can be used to set
    /// furnace to implement your server's middleware stack with custom behavior.
    /// Most users will not need to touch `furnace`. This should only be used if you
    /// need custom handling of `Middleware`. Normally, the default `StackFurnace` is
    /// sufficient.
    pub furnace: F,
    ip: Option<IpAddr>,
    port: Option<u16>
}

impl<F: Clone> Clone for Iron<F> {
    fn clone(&self) -> Iron<F> {
        Iron {
            furnace: self.furnace.clone(),
            ip: self.ip.clone(),
            port: self.port
        }
    }
}

impl<F: Furnace> Iron<F> {
    /// `smelt` a new `Middleware`.
    ///
    /// Adds `Middleware` to the `Iron's` `furnace`, so that any requests
    /// are passed through those `Middleware`.
    ///
    /// `Iron.smelt` delegates to `Iron.furnace.smelt`, so that any `Middleware`
    /// added is added to the `Iron` instance's `furnace`.
    pub fn smelt<M: Middleware>(&mut self, middleware: M) {
        self.furnace.smelt(middleware);
    }

    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This is a blocking operation, and is the final op that should be called
    /// on the `Iron` instance. Once `listen` is called, requests will be
    /// handled as defined through the `Iron's` `furnace's` `Middleware`.
    pub fn listen(mut self, ip: IpAddr, port: u16) {
        self.ip = Some(ip);
        self.port = Some(port);
        self.serve_forever();
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server.
    /// This creates an `Iron` with a default `furnace`, the `StackFurnace`.
    ///
    /// Custom furnaces can be used with `from_furnace`, instead of `new`.
    #[inline]
    pub fn new() -> Iron<F> {
        Iron {
            furnace: Furnace::new(),
            ip: None,
            port: None
        }
    }

    /// Instantiate a new instance of `Iron` from an existing `Furnace`.
    ///
    /// This will create a new `Iron` from a give `Furnace`.
    ///
    /// This `Furnace` *may already have `Middleware` in it*. An empty default
    /// `Furnace` can be created more easily using `new`.
    ///
    /// The `Furnace` can also be configured to handle `Middleware` differently than
    /// `StackFurnace`. For example, this can be used to implement a `Furnace`
    /// that logs debug messages as it serves requests.
    ///
    /// Most users will not need to touch `from_furnace`. This should only be
    /// used if you need custom handling of `Middleware`.
    pub fn from_furnace<F>(furnace: F) -> Iron<F> {
        Iron {
            furnace: furnace,
            ip: None,
            port: None
        }
    }
}

/// Unused, but required for internal functionality.
///
/// This `impl` allows `Iron` to be used as a `Server` by
/// [rust-http]('https://github.com/chris-morgan/rust-http').
/// This is not used by users of this library.
impl<F: Furnace> Server for Iron<F> {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr {
            ip: self.ip.unwrap(),
            port: self.port.unwrap()
        } }
    }

    fn handle_request(&self, req: &Request, res: &mut Response) {
        let mut furnace = self.furnace.clone();
        let _ = furnace.forge(&mut copy_request(req), res, None);
    }
}

// Makes up for no Clone impl on Request objects.
fn copy_request(req: &Request) -> Request {
    Request {
        remote_addr: req.remote_addr,
        headers: req.headers.clone(),
        body: req.body.clone(),
        method: req.method.clone(),
        request_uri: match req.request_uri {
            Star => Star,
            AbsoluteUri(ref u) => AbsoluteUri(u.clone()),
            AbsolutePath(ref p) => AbsolutePath(p.clone()),
            Authority(ref s) => Authority(s.clone())
        },
        close_connection: req.close_connection,
        version: (1, 1)
    }
}

