extern crate iron;

use std::io::net::ip::Ipv4Addr;
use iron::{Iron, Request, Response, IronResult, Set};
use iron::response::modifiers::{Status, Body};
use iron::status;

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::new().set(Status(status::Ok)).set(Body("Hello world!")))
}

fn main() {
    Iron::new(hello_world).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    println!("On 3000");
}

