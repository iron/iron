extern crate iron;

use std::time::Duration;

use iron::prelude::*;
use iron::status;
use iron::Protocol;
use iron::Timeouts;

fn main() {
    Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello world!")))
    })
    .listen_with(
        "localhost:3000",
        8, // thread num
        Protocol::Http,
        Some(
            Timeouts{
                keep_alive: Some(Duration::from_secs(10)),
                read: Some(Duration::from_secs(10)),
                write: Some(Duration::from_secs(10))
            }))
    .unwrap();
}

