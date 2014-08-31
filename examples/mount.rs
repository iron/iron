extern crate iron;
extern crate mount;

use std::io::net::ip::Ipv4Addr;

use iron::status;
use iron::{Iron, Request, Response, IronResult};
use mount::Mount;

fn send_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {}", req.url.path);
    Ok(Response::with(status::Ok, "Hello!"))
}

fn intercept(req: &mut Request) -> IronResult<Response> {
    println!("Running intercept handler, URL path: {}", req.url.path);
    Ok(Response::with(status::Ok, "Blocked!"))
}

fn main() {
    let mut mount = Mount::new();
    mount.mount("/blocked/", intercept).mount("/", send_hello);

    Iron::new(mount).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

