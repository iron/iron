use std::io::net::ip::{SocketAddr, IpAddr};
use std::mem;

use http::server::{Server, Config};
use http::server;

use super::ingot::Ingot;
use super::furnace::Furnace;
use super::response::Response;
use super::request::Request;

use super::response::ironresponse::IronResponse;
use super::request::ironrequest::IronRequest;
use super::furnace::ironfurnace::IronFurnace;

pub type ServerT =
    Iron<IronRequest, IronResponse<'static, 'static>,
         IronFurnace<IronRequest, IronResponse<'static, 'static>>>;

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

impl<Rq: Request, Rs: Response, F: Furnace<Rq, Rs>>
        Iron<Rq, Rs, F> {
    pub fn smelt<I: Ingot<Rq, Rs>>(&mut self, ingot: I) {
        self.furnace.smelt(ingot);
    }

    pub fn listen(mut self, ip: IpAddr, port: u16) {
        self.ip = Some(ip);
        self.port = Some(port);
        self.serve_forever();
    }

    pub fn new<Rq, Rs>() -> Iron<Rq, Rs, F> {
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

impl<Rq: Request,
     Rs: Response,
     F: Furnace<Rq, Rs>>
        Server for Iron<Rq, Rs, F> {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr {
            ip: self.ip.unwrap(),
            port: self.port.unwrap()
        } }
    }

    fn handle_request(&self, req: &server::Request, res: &mut server::ResponseWriter) {
        let request = &mut Request::from_http(req);
        // TODO/FIXME: Replace unsafe block
        let response: &mut Rs = unsafe { mem::transmute(&mut IronResponse::from_http(res)) };
        let mut furnace = self.furnace.clone();
        furnace.forge(request, response, None);
    }
}
