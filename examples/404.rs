extern crate iron;

use iron::prelude::*;
use iron::status;

fn main() {
    Iron::new_sync(|_: &mut Request| {
        Ok(Response::with(status::NotFound))
    }).http("localhost:3000").unwrap();
}

