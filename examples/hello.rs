#![feature(globs)]
extern crate iron;

use iron::prelude::*;
use iron::status;

fn main() {
    Iron::new(|&: _: &mut Request| {
        Ok(Response::new().set(status::Ok).set("Hello world!"))
    }).listen("localhost:3000").unwrap();
}

