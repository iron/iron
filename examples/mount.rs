extern crate iron;
extern crate http;
extern crate mount;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, Request, Response, Alloy, ServerT};
use iron::middleware::{Status, Continue, Unwind};
use iron::mixin::Serve;
use http::status;
use mount::Mount;

fn intercept(_req: &mut Request, _res: &mut Response,
             _alloy: &mut Alloy) -> Status {
    Unwind
}

fn send_hello(_req: &mut Request, res: &mut Response,
              _alloy: &mut Alloy) -> Status {
    let _ = res.serve(status::Ok, "Hello!");
    Continue
}

fn main() {
    let mut server: ServerT = Iron::new();
    server.link(Mount::new("/blocked", intercept));
    server.link(send_hello);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

