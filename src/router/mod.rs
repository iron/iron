use regex::Regex;
use http::method::Method;
use iron::{Middleware, Request, Response, Alloy, Status, Continue, Unwind};
use iron::mixin::GetUrl;

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
    fn enter(&mut self, req: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
        let request_uri = match req.url() {
            Some(uri) => uri.clone(),
            // Not an AbsolutePath, not our problem.
            None => { return Continue; }
        };

        for route in self.routes.mut_iter() {
            if route.method == req.method && route.matches.is_match(request_uri.as_slice()) {
                alloy.insert::<params::Params>(
                    params::Params::new(
                        request_uri.as_slice(),
                        route.matches.clone(),
                        route.params.clone().move_iter()
                    )
                );
                let _ = route.handler.enter(req, res, alloy);
                let _ = route.handler.exit(req, res, alloy);
                return Unwind;
            }
        }
        Continue
    }
}

