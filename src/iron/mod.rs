use std::io::net::ip::{SocketAddr, IpAddr};

// use HttpRequest = http::server::request::Request;
// use HttpResponse = http::server::response::ResponseWriter;
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

    pub fn listen(self) {
        self.serve_forever();
    }

    pub fn new<'a, Rq, Rs>(ip: IpAddr, port: u16) -> Iron<Rq, Rs, F> {
        let furnace = Furnace::new();
        Iron {
            furnace: furnace,
            ip: ip,
            port: port
        }
    }

    pub fn from_furnace<Rq, Rs, F>(furnace: F, ip: IpAddr, port: u16) -> Iron<Rq, Rs, F> {
        Iron {
            furnace: furnace,
            ip: ip,
            port: port
        }
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

