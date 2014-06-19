use regex::Regex;
use http::method::Method;
use iron::{Middleware, Request, Response, Alloy};
use iron::middleware::{Status, Continue, Unwind};
use iron::request::GetUrl;
// Waiting on upstream changes.
//use iron::mixins::{GetUrl};

pub mod params;
mod glob;

pub type Handler = fn(&mut Request, &mut Response, &mut Alloy) -> ();

#[deriving(Clone)]
pub struct Router {
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
    pub fn new() -> Router { Router { routes: Vec::new() } }
    pub fn route(&mut self, method: Method, glob: String,
                 params: Vec<String>, handler: Handler) {
        self.routes.push(Route {
            method: method,
            glob: glob.clone(),
            matches: glob::deglob(glob),
            handler: handler,
            params: params
        });
    }
}

impl Middleware for Router {
    fn enter(&mut self, req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
        let request_uri = match req.url() {
            Some(uri) => uri.clone(),
            // Not an AbsolutePath, not our domain.
            None => { return Continue; }
        };

        for route in self.routes.iter() {
            if route.matches.is_match(request_uri.as_slice()) {
                alloy.insert::<params::Params>(
                    params::Params::new(
                        request_uri.as_slice(),
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

