//! Exposes the `Iron` type, the main entrance point of the
//! `Iron` library.

use std::io::net::ip::{SocketAddr, IpAddr};

use http::server::{Server, Config};
use http::server;

use super::ingot::Ingot;
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
/// `Iron.smelt` is used to add new `Ingot`s, and
/// `Iron.listen` is used to kick off a server process.
///
/// `Iron` contains the `Furnace` which holds the `Ingot`s necessary to run a server.
/// `Iron` is the main interface to adding `Ingot`s, and has `Furnace` as a
/// public field (for the sake of extensibility).
pub struct Iron<Rq, Rs, F> {
    /// The exposed internal field for storing `Furnace`.
    ///
    /// This is exposed for the sake of extensibility. It can be used to set
    /// furnace to implement your server's middleware stack with custom behavior.
    /// Most users will not need to touch `furnace`. This should only be used if you
    /// need custom handling of `Ingot`s. Normally, the default `IronFurnace` is
    /// sufficient.
    pub furnace: F,
    ip: Option<IpAddr>,
    port: Option<u16>
}

impl<Rq, Rs, F: Clone> Clone for Iron<Rq, Rs, F> {
    fn clone(&self) -> Iron<Rq, Rs, F> {
        Iron {
            furnace: self.furnace.clone(),
            ip: self.ip.clone(),
            port: self.port
        }
    }
}

impl<'a, 'b,
     Rq: Request + HttpRequest,
     Rs: Response + HttpResponse<'a, 'b>,
     F: Furnace<Rq, Rs>>
        Iron<Rq, Rs, F> {
    /// `smelt` a new `Ingot`.
    ///
    /// This adds an `Ingot` to the `Iron`'s `furnace`, so that any requests
    /// are passed through those `Ingot`s.
    ///
    /// `Iron.smelt` delegates to `iron.furnace.smelt`, so that any `Ingot`
    /// added is added to the `Iron` instance's `furnace`.
    pub fn smelt<I: Ingot<Rq, Rs>>(&mut self, ingot: I) {
        self.furnace.smelt(ingot);
    }

    /// Kick off the server process.
    ///
    /// Call this once to begin listening for requests on the server.
    /// This is a blocking operation, and is the final op that should be called
    /// on the `Iron` instance. Once `listen` is called, requests will be
    /// handled as defined through the `Iron`'s `furnace`'s `Ingot`s.
    pub fn listen(mut self, ip: IpAddr, port: u16) {
        self.ip = Some(ip);
        self.port = Some(port);
        self.serve_forever();
    }

    /// Instantiate a new instance of `Iron`.
    ///
    /// This will create a new `Iron`, the base unit of the server.
    /// This creates an `Iron` with a default `furnace`, the `IronFurnace`.
    ///
    /// Custom furnaces can be used with `from_furnace`, instead of `new`.
    #[inline]
    pub fn new<Rq, Rs>() -> Iron<Rq, Rs, F> {
        let furnace = Furnace::new();
        Iron {
            furnace: furnace,
            ip: None,
            port: None
        }
    }

    /// Instantiate a new instance of `Iron` from an existing `Furnace`.
    ///
    /// This will create a new `Iron` from a give `Furnace`.
    ///
    /// This `Furnace` *may already have `Ingot`s in it*. An empty default
    /// `Furnace` can be created more easily using `new`.
    ///
    /// The `Furnace` can also be configured to handle `Ingot`s differently than
    /// `IronFurnace`. For example, this can be used to implement a `Furnace`
    /// that logs debug messages as it serves requests.
    ///
    /// Most users will not need to touch `from_furnace`. This should only be
    /// used if you need custom handling of `Ingot`s. Normally, the default
    /// `IronFurnace` is sufficient.
    pub fn from_furnace<Rq, Rs, F>(furnace: F) -> Iron<Rq, Rs, F> {
        Iron {
            furnace: furnace,
            ip: None,
            port: None
        }
    }
}

/// This is unused but required for internal functionality.
///
/// This `impl` allows `Iron` to be used as a `Server` by
/// [rust-http]('https://github.com/chris-morgan/rust-http').
/// This is not used by users of this library.
impl<'a, 'b,
     Rq: Request + HttpRequest,
     Rs: Response + HttpResponse<'a, 'b>,
     F: Furnace<Rq, Rs>>
        Server for Iron<Rq, Rs, F> {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr {
            ip: self.ip.unwrap(),
            port: self.port.unwrap()
        } }
    }

    fn handle_request(&self, req: &server::Request, res: &mut server::ResponseWriter) {
        let mut furnace = self.furnace.clone();
        handler::<'a, 'b, Rq, Rs, F>(&mut furnace, req, res);
    }
}

fn handler<'a, 'b,
            Rq: Request + HttpRequest,
            Rs: Response + HttpResponse<'a, 'b>,
            F: Furnace<Rq, Rs>>
        (furnace: &mut F, req: &server::Request, res: &mut server::ResponseWriter) {
    let mut request: Rq = HttpRequest::from_http(req);
    let mut response: Rs = HttpResponse::from_http(res);
    furnace.forge(&mut request, &mut response, None);
}
