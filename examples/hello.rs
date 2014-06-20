extern crate http;
extern crate iron;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, Alloy, Request, Response, ServerT};
use iron::mixin::Serve;
use iron::middleware::{Status, Continue};

fn hello_world(_req: &mut Request, res: &mut Response, _alloy: &mut Alloy) -> Status {
    let _ = res.serve(::http::status::Ok, "Hello, world!");
    Continue
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.smelt(hello_world);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

