#![feature(phase)]

extern crate iron;
extern crate http;
#[phase(plugin, link)]
extern crate mount;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, Middleware, Request, Response, Alloy, ServerT};
use iron::middleware::{Status, Continue, Unwind};
use iron::mixin::Serve;
use http::status;

#[deriving(Clone)]
struct Intercept;

impl Middleware for Intercept {
    fn enter(&mut self,
             _req: &mut Request,
             _res: &mut Response,
             _alloy: &mut Alloy) -> Status {
        Unwind
    }
}

#[deriving(Clone)]
struct SendHello;

impl Middleware for SendHello {
    fn enter(&mut self,
             _req: &mut Request,
             res: &mut Response,
             _alloy: &mut Alloy) -> Status {
        let _ = res.serve(status::Ok, "Hello!");
        Continue
    }
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.smelt(mount!("/blocked", Intercept));
    server.smelt(SendHello);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

