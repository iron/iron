use std::io::net::ip::{SocketAddr, IpAddr};

use http::server::{Server, Config};
use http::server;

use super::ingot::Ingot;
use super::furnace::Furnace;
use super::response::Response;
use super::request::Request;

#[deriving(Send)]
pub struct Iron<Rq, Rs, F> {
    furnace: F,
    ip: IpAddr,
    port: u16
}

impl<Rq: Clone, Rs: Clone, F: Clone> Clone for Iron<Rq, Rs, F> {
    fn clone(&self) -> Iron<Rq, Rs, F> {
        Iron {
            furnace: self.furnace.clone(),
            ip: self.ip.clone(),
            port: self.port
        }
    }
}

impl<Rq: Request, Rs: Response, F: Furnace<Rq, Rs>>
        Iron<Rq, Rs, F> {
    fn smelt<I: Ingot<Rq, Rs>>(&mut self, _ingot: I) {
        // some stuff
    }

    fn listen(mut self, ip: IpAddr, port: u16) {
        self.ip = ip;
        self.port = port;
        self.serve_forever();
    }
}

impl<Rq: Request,
     Rs: Response,
     F: Furnace<Rq, Rs>>
        Server for Iron<Rq, Rs, F> {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: self.ip, port: self.port } }
    }

    fn handle_request(&self, _req: &server::Request, _res: &mut server::ResponseWriter) {
        // coerce allllllll that
    }
}

