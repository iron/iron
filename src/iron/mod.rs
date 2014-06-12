use std::io::net::ip::{SocketAddr, IpAddr};

use http::server::{Server, Config};
use http::server;

use super::middleware::{Middleware, MiddlewareStack};

trait Request: Send + Clone {}
trait Response: Send + Clone {}

#[deriving(Send)]
pub struct Iron<Rq, Rs, Ms> {
    mid_stack: Ms,
    ip: IpAddr,
    port: u16
}

impl<Rq: Clone, Rs: Clone, Ms: Clone> Clone for Iron<Rq, Rs, Ms> {
    fn clone(&self) -> Iron<Rq, Rs, Ms> {
        Iron {
            mid_stack: self.mid_stack.clone(),
            ip: self.ip.clone(),
            port: self.port
        }
    }
}

impl<Rq: Request, Rs: Response, Ms: MiddlewareStack<Rq, Rs>>
        Iron<Rq, Rs, Ms> {
    fn smelt<M: Middleware<Rq, Rs>>(&mut self, _ingot: M) {
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
     Ms: MiddlewareStack<Rq, Rs>>
        Server for Iron<Rq, Rs, Ms> {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: self.ip, port: self.port } }
    }

    fn handle_request(&self, _req: &server::Request, _res: &mut server::ResponseWriter) {
        // coerce allllllll that
    }
}

