use regex::Regex;
use http::method::{Method, Options};
use http::status::{InternalServerError};
use iron::{Middleware, Request, Response, Alloy};
use iron::middleware::{Status, Continue, Unwind};

pub mod params;
mod glob;

pub type Handler = fn(&mut Request, &mut Response, &mut Alloy) -> ();

#[deriving(Clone)]
pub struct Router {
    options: Vec<Method>,
    routes: Vec<Route>
}

struct Route {
    method: Method,
    glob: String,
    matches: Regex,
    handler: Handler,
    params: Vec<String>
}

impl Clone for Route {
    fn clone(&self) -> Route {
        Route {
            method: self.method.clone(),
            glob: self.glob.clone(),
            matches: self.matches.clone(),
            handler: self.handler,
            params: self.params.clone()
        }
    }
}

impl Router {
    pub fn new() -> Router { Router { options: Vec::new(), routes: Vec::new() } }
    pub fn route(&mut self, method: Method, glob: String,
                 params: Vec<String>, handler: Handler) {
        self.add_route(Route {
            method: method,
            glob: glob.clone(),
            matches: glob::deglob(glob),
            handler: handler,
            params: params
        });
    }

    fn add_route(&mut self, route: Route) {
        if !self.options.contains(&route.method) {
            self.options.push(route.method.clone())
        }
        self.routes.push(route);
    }
}

impl Middleware for Router {
    fn enter(&mut self, req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
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

