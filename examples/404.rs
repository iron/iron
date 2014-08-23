
extern crate http;
extern crate iron;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, Request, Response, IronResult};
use iron::status;

fn fourzerofour(_: &mut Request) -> IronResult<Response> {
    let mut res = Response::new();
    res.status = Some(status::NotFound);
    Ok(res)
}

fn main() {
    Iron::new(fourzerofour).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    println!("On 3k");
}

