extern crate iron;
extern crate mount;

use iron::{Iron, Request, Response, IronResult, Set};
use iron::response::modifiers::{Body, Status};
use iron::status;

use mount::Mount;

fn send_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {}", req.url.path);
    Ok(Response::new().set(Status(status::Ok)).set(Body("Hello!")))
}

fn intercept(req: &mut Request) -> IronResult<Response> {
    println!("Running intercept handler, URL path: {}", req.url.path);
    Ok(Response::new().set(Status(status::Ok)).set(Body("Blocked!")))
}

fn main() {
    let mut mount = Mount::new();
    mount.mount("/blocked/", intercept).mount("/", send_hello);

    Iron::new(mount).listen("localhost:3000").unwrap();
}

