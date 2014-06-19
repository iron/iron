extern crate http;
extern crate iron;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, Alloy, Middleware, Request, Response, ServerT};
use iron::mixin::Serve;
use iron::middleware::{Status, Continue};

#[deriving(Clone)]
struct HelloWorld;

impl Middleware for HelloWorld {
    fn enter(&mut self,
             _request: &mut Request,
             response: &mut Response,
             _alloy: &mut Alloy) -> Status {
        let _ = response.serve(::http::status::Ok, "Hello, world!");
        Continue
    }
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.smelt(HelloWorld);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

