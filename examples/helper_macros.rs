//! A simple demonstration how iron's helper macros make e.g. IO-intensive code easier to write.
#[macro_use]
extern crate iron;

use std::fs;
use std::io;
use std::io::Cursor;

use iron::prelude::*;
use iron::Method;
use iron::StatusCode;

fn main() {
    Iron::new(|req: &mut Request| {
        Ok(match req.method {
            Method::GET => {
                // It's not a server error if the file doesn't exist yet. Therefore we use
                // `iexpect`, to return Ok(...) instead of Err(...) if the file doesn't exist.
                let f = iexpect!(fs::File::open("foo.txt").ok(), (StatusCode::OK, ""));
                Response::with((StatusCode::OK, f))
            }
            Method::PUT => {
                // If creating the file fails, something is messed up on our side. We probably want
                // to log the error, so we use `itry` instead of `iexpect`.
                let mut f = itry!(fs::File::create("foo.txt"));
                itry!(io::copy(
                    &mut Cursor::new(itry!(req.get_body_contents())),
                    &mut f
                ));
                Response::with(StatusCode::CREATED)
            }
            _ => Response::with(StatusCode::BAD_REQUEST),
        })
    }).http("localhost:3000");
}
