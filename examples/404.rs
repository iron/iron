#![feature(globs)]
extern crate iron;

use iron::prelude::*;
use iron::response::modifiers::Status;
use iron::status;

fn fourzerofour(_: &mut Request) -> IronResult<Response> {
    Ok(Response::new()
           .set(Status(status::NotFound)))
}

fn main() {
    Iron::new(fourzerofour).listen("localhost:3000").unwrap();
    println!("On 3000");
}

