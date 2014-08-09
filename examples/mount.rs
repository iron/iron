extern crate iron;
extern crate http;
extern crate mount;

use std::io::net::ip::Ipv4Addr;

use http::status;
use iron::{Iron, Request, Response, Server, Chain, Status, Continue, Unwind, FromFn};
use mount::Mount;

fn intercept(_: &mut Request, res: &mut Response) -> Status {
    let _ = res.serve(status::Ok, "Blocked!");
    Unwind
}

fn send_hello(_: &mut Request, res: &mut Response) -> Status {
    let _ = res.serve(status::Ok, "Hello!");
    Continue
}

fn main() {
    let mut server: Server = Iron::new();
    server.chain.link(Mount::new("/blocked/", FromFn::new(intercept)));
    server.chain.link(FromFn::new(send_hello));
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

