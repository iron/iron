extern crate iron;
extern crate mount;

use std::io::net::ip::Ipv4Addr;

use iron::status;
use iron::{Iron, Request, Response, IronResult, Url};
use mount::{Mount, OriginalUrl};

fn level_two(req: &mut Request) -> IronResult<Response> {
    match req.extensions.find::<OriginalUrl, Url>() {
        Some(url) => println!("Original URL: {}", url),
        None => println!("Error: No original URL found.")
    }
    Ok(Response::with(status::Ok, "Welcome to Level 2."))
}

fn main() {
    let mut first = Mount::new();
    let mut second = Mount::new();
    second.mount("/leveltwo/", level_two);
    first.mount("/levelone/", second);

    Iron::new(first).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

