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
    method!(get, Get)
    method!(post, Post)
    method!(put, Put)
    method!(patch, Patch)
    method!(delete, Delete)
}

macro_rules! method {
    ($name:ident, $method:ident) => {
        fn $name(&mut self, matches: Regex, handler: Handler) {
            self.addRoute(Route {
                method: $method
                matches: matches,
                handler: handler
            });
        }
    }
}

impl<Rq: Request, Rs: Response> Ingot<Rq, Rs> for Router {
    fn enter(&mut self, req: &mut Rq, res: &mut Rs, alloy: &mut Alloy) -> Status {
        if *req.method() == Options {
            res.write(self.options.iter().map(|p| p.show()).collect().join(" "));
            return Unwind;
        }
        for route in self.routes.iter() {
            if route.matches.is_match(req.reques_uri()) {
                route.handler(req, res, alloy);
                return Unwind;
            }
        }
        Continue
    }
}

