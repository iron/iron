extern crate iron;

use iron::prelude::*;
use iron::modifiers::Redirect;
use iron::{Url, status};

fn main() {
    let url = Url::parse("http://rust-lang.org").unwrap();

    Iron::new(move |_: &mut Request | {
        Ok(Response::with((status::Found, Redirect(url.clone()))))
    }).http("localhost:3000").unwrap();
}

