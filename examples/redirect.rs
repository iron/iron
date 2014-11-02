extern crate iron;

use std::io::net::ip::Ipv4Addr;
use iron::{Iron, Request, Response, IronResult, Url, Set};
use iron::response::modifiers::{Status, Redirect};
use iron::status;

fn redirect(_: &mut Request) -> IronResult<Response> {
    let url = Url::parse("http://rust-lang.org").unwrap();
    Ok(Response::new()
           .set(Status(status::Ok))
           .set(Redirect(url)))
}

fn main() {
    Iron::new(redirect).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    println!("On 3000");
}

