extern crate http;
extern crate iron;

use std::io::net::ip::Ipv4Addr;
use iron::{Iron, Request, Response, IronResult};
use iron::status;

fn hello_world(_: &mut Request) -> IronResult<Response> {
    let mut res = Response::new();
    let _ = res.serve(status::Ok, "Hello, world!");
    Ok(res)
}

fn main() {
    Iron::new(hello_world).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    println!("On 3000");
}

