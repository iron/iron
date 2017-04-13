extern crate iron;

use iron::prelude::*;
use iron::status;

fn main() {
    Iron::new(|_: &mut Request| Ok(Response::with((status::Ok, "Hello world!"))))
        .http("localhost:3000")
        .unwrap();
}
