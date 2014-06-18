use regex::Regex;
use http::method::{Method, Options};
use http::status::{InternalServerError};
use iron::{Ingot, Request, Response, Alloy};
use iron::ingot::{Status, Continue, Unwind};

pub mod params;
mod glob;

pub type Handler<Rq, Rs> = fn(&mut Rq, &mut Rs, &mut Alloy) -> ();

pub struct Router<Rq, Rs> {
    options: Vec<Method>,
    routes: Vec<Route<Rq, Rs>>
}

impl<Rq, Rs> Clone for Router<Rq, Rs> {
    fn clone(&self) -> Router<Rq, Rs> {
        Router {
            options: self.options.clone(),
            routes: self.routes.clone()
        }
    }
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
            matches: glob::deglob(glob.clone()),
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

impl<Rq: Request, Rs: Response> Ingot<Rq, Rs> for Router<Rq, Rs> {
    fn enter(&mut self, req: &mut Rq, res: &mut Rs, alloy: &mut Alloy) -> Status {
        if *req.method() == Options {
            match res.write(
                self.options.iter()
                    .map(|p| format!("{}", p))
                    .collect::<Vec<String>>()
                    .connect(" ").as_bytes()) {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to write response: {}", err);
                    *res.status_mut() = InternalServerError;
                }
            }
            return Unwind;
        }
        for route in self.routes.iter() {
            if route.matches.is_match(format!("{}", req.uri()).as_slice()) {
                alloy.insert::<params::Params>(
                    params::Params::new(
                        format!("{}", req.uri()).as_slice(),
                        route.matches.clone(),
                        route.params.clone().move_iter()
                    )
                );
                (route.handler)(req, res, alloy);
                return Unwind;
            }
        }
        Continue
    }
}

