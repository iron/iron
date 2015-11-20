//! A simple demonstration how iron's helper macros make e.g. IO-intensive code easier to write.
#[macro_use] extern crate iron;

use std::io;
use std::fs;

use iron::prelude::*;
use iron::status;
use iron::method;

fn main() {
    Iron::new(|req: &mut Request| {
        Ok(match req.method {
            method::Get => {
                // It's not a server error if the file doesn't exist yet. Therefore we use
                // `iexpect`, to return Ok(...) instead of Err(...) if the file doesn't exist.
                let f = iexpect!(fs::File::open("foo.txt").ok(), (status::Ok, ""));
                Response::with((status::Ok, f))
            },
            method::Put => {
                // If creating the file fails, something is messed up on our side. We probably want
                // to log the error, so we use `itry` instead of `iexpect`.
                let mut f = itry!(fs::File::create("foo.txt"));
                itry!(io::copy(&mut req.body, &mut f));
                Response::with(status::Created)
            },
            _ => Response::with(status::BadRequest)
        })
    }).http("localhost:3000").unwrap();
}

