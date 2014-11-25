extern crate iron;
extern crate mount;

use iron::{Iron, Request, Response, IronResult, Url, Set};
use iron::response::modifiers::{Body, Status};
use iron::status;

use mount::{Mount, OriginalUrl};

fn level_two(req: &mut Request) -> IronResult<Response> {
    match req.extensions.get::<OriginalUrl, Url>() {
        Some(url) => println!("Original URL: {}", url),
        None => println!("Error: No original URL found.")
    }
    Ok(Response::new().set(Status(status::Ok)).set(Body("Welcome to Level 2.")))
}

fn main() {
    let mut first = Mount::new();
    let mut second = Mount::new();
    second.mount("/leveltwo/", level_two);
    first.mount("/levelone/", second);

    Iron::new(first).listen("localhost:3000").unwrap();
}

