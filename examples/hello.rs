extern crate iron;

use iron::prelude::*;
use iron::status::StatusCode;

fn main() {
    Iron::new(|_: &mut Request| {
        Ok(Response::with((StatusCode::OK, "Hello world!")))
    }).http("localhost:3000");
}

