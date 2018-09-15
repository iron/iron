extern crate iron;
extern crate mount;

use iron::{Iron, Request, Response, IronResult, StatusCode};
use mount::Mount;

fn send_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {:?}", req.url.path());
    Ok(Response::with((StatusCode::OK, "Hello!")))
}

fn intercept(req: &mut Request) -> IronResult<Response> {
    println!("Running intercept handler, URL path: {:?}", req.url.path());
    Ok(Response::with((StatusCode::OK, "Blocked!")))
}

fn main() {
    let mut mount = Mount::new();
    mount.mount("/blocked/", intercept).mount("/", send_hello);

    Iron::new(mount).http("localhost:3000");
}
