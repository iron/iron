extern crate iron;
extern crate mount;

use iron::{Iron, Request, Response, IronResult};
use iron::status;

use mount::{Mount, OriginalUrl};

fn level_two(req: &mut Request) -> IronResult<Response> {
    match req.extensions.get::<OriginalUrl>() {
        Some(url) => println!("Original URL: {}", url),
        None => println!("Error: No original URL found.")
    }
    Ok(Response::with((status::Ok, "Welcome to Level 2.")))
}

fn main() {
    let mut first = Mount::new();
    let mut second = Mount::new();
    second.mount("/leveltwo/", level_two);
    first.mount("/levelone/", second);

    Iron::new(first).http("localhost:3000").unwrap();
}

