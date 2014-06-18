extern crate iron;

use std::io::net::ip::Ipv4Addr;

use iron::{Ingot, Iron, Alloy, Request, Response, ServerT};
use iron::ingot::{Status, Continue};

#[deriving(Clone)]
struct HelloWorld;

impl Ingot for HelloWorld {
    fn enter(&mut self,
             _request: &mut Request,
             response: &mut Response,
             _alloy: &mut Alloy) -> Status {
        let _ = response.write(bytes!("Hello World!"));
        Continue
    }
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.smelt(HelloWorld);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

