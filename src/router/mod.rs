use std::fmt::Show;
use regex::Regex;
use http::method::Method;
use iron::{Middleware, Request, Response, Status, Continue, Unwind, Error};

pub mod params;
mod glob;

/// `Router` provides an interface for creating complex routes as middleware
/// for the Iron framework.
#[deriving(Clone)]
pub struct Router {
    routes: Vec<Route>
}

struct Route {
    method: Method,
    glob: String,
    matches: Regex,
    handler: Box<Middleware + Send>,
    params: Vec<String>
}

impl Clone for Route {
    fn clone(&self) -> Route {
        Route {
            method: self.method.clone(),
            glob: self.glob.clone(),
            matches: self.matches.clone(),
            handler: self.handler.clone_box(),
            params: self.params.clone()
        }
    }
}

impl Router {
    /// `new` constructs a new, blank `Router`.
    pub fn new() -> Router { Router { routes: Vec::new() } }
    /// Add a new route to a `Router`, matching both a method and glob pattern.
    ///
    /// `route` supports express-style glob patterns: `*` for a single wildcard
    /// segment, `**` for any-number of wildcard segments, and `:param` for
    /// matching storing that segment of the request url in the `Params`
    /// object, which is stored on an Alloy.
    ///
    /// For instance, to route `Get` requests on any route matching
    /// `/users/:userid/:friend` and store `userid` and `friend` in
    /// the exposed Params object:
    ///
    /// ```rust
    /// router.route(
    ///     ::http::method::Get,
    ///     "/users/:userid/:friend".to_string(),
    ///     controller
    /// ```
    ///
    /// The controller provided to route can be any `Middleware`, which allows
    /// extreme flexibility when handling routes. For instance, you could provide
    /// a `Chain`, a `Middleware`, which contains an authorization middleware and
    /// a controller function, so that you can confirm that the request is
    /// authorized for this route before handling it.
    pub fn route<M: Middleware + Send>(&mut self, method: Method, glob: String,
                                       params: Vec<String>, handler: M) {
        self.routes.push(Route {
            method: method,
            glob: glob.clone(),
            matches: glob::deglob(glob),
            handler: box handler,
            params: params
        });
    }
}

impl Middleware for Router {
    fn enter(&mut self, req: &mut Request, res: &mut Response) -> Status {
        let url_path = "/".to_string().append(match req.url.path() {
            Some(paths) => paths.connect("/"),
            None => return Continue
        }.as_slice());

        for route in self.routes.mut_iter() {
            if route.method == req.method && route.matches.is_match(url_path.as_slice()) {
                req.alloy.insert::<params::Params>(
                    params::Params::new(
                        url_path.as_slice(),
                        route.matches.clone(),
                        route.params.clone().move_iter()
                    )
                );
                let mut handler_status = route.handler.enter(req, res);

                match handler_status {
                    Error(ref mut e) => {
                        let error: &mut Show = *e;
                        let _ = route.handler.on_error(req, res, error);
                    },
                    _ => {
                        let _ = route.handler.exit(req, res);
                    }
                };

                return match handler_status {
                    Continue => Unwind,
                    otherwise => otherwise
                }
            }
        }

        Continue
    }
}

