#![feature(globs)]
extern crate iron;

use iron::prelude::*;
use iron::status;

fn main() {
    Iron::new(|&: _: &mut Request| {
        Ok(Response::with(status::NotFound))
    }).listen("localhost:3000").unwrap();
    println!("On 3000");
}

