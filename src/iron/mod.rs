use std::io::net::ip::{SocketAddr, IpAddr};

use http::server::{Server, Config};
use http::server;

use super::ingot::Ingot;
use super::furnace::Furnace;
use super::response::Response;
use super::request::Request;

pub struct Iron<Rq, Rs, F> {
    pub furnace: F,
    ip: IpAddr,
    port: u16
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

impl<'a, Rq: Request, Rs: Response<'a>, F: Furnace<'a, Rq, Rs>>
        Iron<Rq, Rs, F> {
    pub fn smelt<I: Ingot<'a, Rq, Rs>>(&mut self, ingot: I) {
        self.furnace.smelt(ingot);
    }

    pub fn listen(mut self, ip: IpAddr, port: u16) {
        self.ip = ip;
        self.port = port;
        self.serve_forever();
    }
}

impl<'a,
     Rq: Request,
     Rs: Response<'a>,
     F: Furnace<'a, Rq, Rs>>
        Server for Iron<Rq, Rs, F> {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: self.ip, port: self.port } }
    }

    fn handle_request(&self, _req: &server::Request, _res: &mut server::ResponseWriter) {
        // coerce allllllll that
    }
}

