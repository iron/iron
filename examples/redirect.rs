#![feature(globs)]
extern crate iron;

use iron::prelude::*;
use iron::response::modifiers::{Status, Redirect};
use iron::{Url, status};

fn redirect(_: &mut Request) -> IronResult<Response> {
    let url = Url::parse("http://rust-lang.org").unwrap();
    Ok(Response::new()
           .set(Status(status::Ok))
           .set(Redirect(url)))
}

fn main() {
    Iron::new(redirect).listen("localhost:3000").unwrap();
    println!("On 3000");
}

