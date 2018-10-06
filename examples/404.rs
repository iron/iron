extern crate iron;

use iron::status;
use iron::prelude::*;

fn main() {
    Iron::new(|_: &mut Request| Ok(Response::with(status::NotFound))).http("localhost:3000");
}
