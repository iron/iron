extern crate iron;

use iron::prelude::*;
use iron::StatusCode;

fn main() {
    Iron::new(|_: &mut Request| Ok(Response::with(StatusCode::NOT_FOUND))).http("localhost:3000");
}
