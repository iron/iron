extern crate iron;

use iron::modifiers::Redirect;
use iron::prelude::*;
use iron::{StatusCode, Url};

fn main() {
    let url = Url::parse("http://rust-lang.org").unwrap();

    Iron::new(move |_: &mut Request| {
        Ok(Response::with((StatusCode::FOUND, Redirect(url.clone()))))
    }).http("localhost:3000");
}
