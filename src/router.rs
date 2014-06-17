use regex::Regex;
use http::method::{Method, Options};
use http::status::{InternalServerError};
use iron::{Ingot, Request, Response, Alloy};
use iron::ingot::{Status, Continue, Unwind};

pub type Handler<Rq, Rs> = fn(&mut Rq, &mut Rs, &mut Alloy) -> ();

#[deriving(Clone)]
pub struct Router<Rq, Rs> {
    options: Vec<Method>,
    routes: Vec<Route<Rq, Rs>>
}

struct Route<Rq, Rs> {
    method: Method,
    glob: String,
    matches: Regex,
    handler: Handler<Rq, Rs>,
    params: Vec<String>
}

impl<Rq, Rs> Clone for Route<Rq, Rs> {
    fn clone(&self) -> Route<Rq, Rs> {
        Route {
            method: self.method.clone(),
            glob: self.glob.clone(),
            matches: self.matches.clone(),
            handler: self.handler,
            params: self.params.clone()
        }
    }
}

impl<Rq, Rs> Router<Rq, Rs> {
    pub fn new() -> Router<Rq, Rs> { Router { options: Vec::new(), routes: Vec::new() } }
    pub fn route(&mut self, method: Method, glob: String,
                 params: Vec<String>, handler: Handler<Rq, Rs>) {
        self.add_route(Route {
            method: method,
            matches: deglob(glob.clone()),
            params: params,
            handler: handler,
            glob: glob
        });
    }

    fn add_route(&mut self, route: Route<Rq, Rs>) {
        if !self.options.contains(&route.method) {
            self.options.push(route.method.clone())
        }
        self.routes.push(route);
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

