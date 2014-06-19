use http::server::request::{AbsolutePath};
use regex::Regex;

use iron::{Iron, Middleware, Request, Response, Alloy, Furnace};
use iron::middleware::{Status, Continue, Unwind};

#[deriving(Clone)]
pub struct Mount<F> {
    route: String,
    matches: Regex,
    iron: Iron<F>
}

impl<F> Mount<F> {
    pub fn new(route: &str, iron: Iron<F>) -> Mount<F> {
        Mount {
            route: route.to_string(),
            iron: iron,
            matches: to_regex(route)
        }
    }
}

fn to_regex(route: &str) -> Regex {
    Regex::new("^".to_string().append(route).as_slice()).unwrap()
}

