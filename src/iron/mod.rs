use std::io::net::ip::{SocketAddr, IpAddr};
use std::mem;

use http::server::{Server, Config};
use http::server;

use super::ingot::Ingot;
use super::furnace::Furnace;
use super::response::Response;
use super::request::Request;

pub struct Iron<Rq, Rs, F> {
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

impl<'a, Rq: Request, Rs: Response<'a>, F: Furnace<'a, Rq, Rs>>
        Iron<Rq, Rs, F> {
    pub fn smelt<I: Ingot<'a, Rq, Rs>>(&mut self, ingot: I) {
        self.furnace.smelt(ingot);
    }

    pub fn listen(mut self, ip: IpAddr, port: u16) {
        self.ip = Some(ip);
        self.port = Some(port);
        self.serve_forever();
    }

    pub fn new<'a, Rq, Rs>() -> Iron<Rq, Rs, F> {
        let furnace = Furnace::new();
        Iron {
            furnace: furnace,
            ip: None,
            port: None
        }
    }

    pub fn from_furnace<Rq, Rs, F>(furnace: F) -> Iron<Rq, Rs, F> {
        Iron {
            furnace: furnace,
            ip: None,
            port: None
        }
    }
}

impl<'a,
     Rq: Request,
     Rs: Response<'a>,
     F: Furnace<'a, Rq, Rs>>
        Server for Iron<Rq, Rs, F> {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr {
            ip: self.ip.unwrap(),
            port: self.port.unwrap()
        } }
    }

    fn handle_request(&self, req: &server::Request, res: &mut server::ResponseWriter) {
        let request = &mut Request::from_http(req);
        let response: &mut Rs = unsafe { mem::transmute(res) };
        let mut furnace = self.furnace.clone();
        furnace.forge(request, response, None);
    }
}
