extern crate iron;

use std::time::Duration;

use iron::prelude::*;
use iron::status;
use iron::Timeouts;

fn main() {
    let mut iron = Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello world!")))
    });
    iron.threads = 8;
    iron.timeouts = Timeouts {
        keep_alive: Some(Duration::from_secs(10)),
        read: Some(Duration::from_secs(10)),
        write: Some(Duration::from_secs(10))
    };
    iron.http("localhost:3000").unwrap();
}

