extern crate iron;

use iron::prelude::*;
use iron::mime::Mime;
use iron::status;


fn main() {
    Iron::new(|_: &mut Request| {
        let content_type = "application/json".parse::<Mime>().unwrap();
        Ok(Response::with((content_type, status::Ok, "{}")))
    }).http("localhost:3000").unwrap();
}
