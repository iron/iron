#![feature(globs)]
extern crate iron;

use iron::prelude::*;
use iron::response::modifiers::Redirect;
use iron::{Url, status};

fn main() {
    Iron::new(|&: _: &mut Request | {
        let url = Url::parse("http://rust-lang.org").unwrap();
        Ok(Response::with((status::Ok, Redirect(url))))
    }).listen("localhost:3000").unwrap();
    println!("On 3000");
}

