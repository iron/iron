use regex::Regex;
use http::{Method, Options, Get, Post, Put, Delete, Patch};
use iron::{Ingot, Request, Response, Alloy};
use iron::ingot::{Status, Continue, Unwind};

pub type Handler = |&mut Rq, &mut Rs, &mut A| -> ()

#[deriving(Clone)]
pub struct Router {
    options: Vec<Method>,
    routes: Vec<Route>
}

#[deriving(Clone)]
struct Route {
    method: Method,
    matches: Regex,
    handler: |&mut Rq, &mut Rs, &mut A| -> ()
}

impl Router {
    fn new() -> Router { Router { options: Vec::new(), routes: Vec::new() } }
    fn addRoute(&mut self, route: Route) {
        if !self.options.contains(route.method) {
            self.options.push(route.method)
        }
        self.routes.push(route);
    }
}

