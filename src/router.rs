use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::error::Error;
use std::fmt;

use iron::{Request, Response, Handler, IronResult, IronError};
use iron::{status, method, headers};
use iron::typemap::Key;
use iron::modifiers::Redirect;

use recognizer::Router as Recognizer;
use recognizer::{Match, Params};

/// `Router` provides an interface for creating complex routes as middleware
/// for the Iron framework.
pub struct Router {
    // The routers, specialized by method.
    routers: HashMap<method::Method, Recognizer<Box<Handler>>>
}

unsafe impl Send for Router {}
unsafe impl Sync for Router {}

impl Router {
    /// Construct a new, empty `Router`.
    ///
    /// ```
    /// # use router::Router;
    /// let router = Router::new();
    /// ```
    pub fn new() -> Router {
        Router {
            routers: HashMap::new()
        }
    }

    /// Add a new route to a `Router`, matching both a method and glob pattern.
    ///
    /// `route` supports glob patterns: `*` for a single wildcard segment and
    /// `:param` for matching storing that segment of the request url in the `Params`
    /// object, which is stored in the request `extensions`.
    ///
    /// For instance, to route `Get` requests on any route matching
    /// `/users/:userid/:friend` and store `userid` and `friend` in
    /// the exposed Params object:
    ///
    /// ```ignore
    /// let mut router = Router::new();
    /// router.route(method::Get, "/users/:userid/:friendid", controller);
    /// ```
    ///
    /// The controller provided to route can be any `Handler`, which allows
    /// extreme flexibility when handling routes. For instance, you could provide
    /// a `Chain`, a `Handler`, which contains an authorization middleware and
    /// a controller function, so that you can confirm that the request is
    /// authorized for this route before handling it.
    pub fn route<H, S>(&mut self, method: method::Method,
                       glob: S, handler: H) -> &mut Router
    where H: Handler, S: Str {
        match self.routers.entry(method) {
            Vacant(entry)   => entry.insert(Recognizer::new()),
            Occupied(entry) => entry.into_mut()
        }.add(glob.as_slice().trim_right_matches('/'),
              Box::new(handler) as Box<Handler>);
        self
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Get, glob, handler)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Post, glob, handler)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Put, glob, handler)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Delete, glob, handler)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Head, glob, handler)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Patch, glob, handler)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Options, glob, handler)
    }

    fn recognize<'a>(&'a self, method: &method::Method, path: &str)
                     -> Option<Match<&'a Box<Handler>>> {
        self.routers.get(method).and_then(|router| router.recognize(path).ok())
    }

    fn handle_options(&self, req: &mut Request, path: &str) -> IronResult<Response> {
        static METHODS: &'static [method::Method] =
            &[method::Get, method::Post, method::Post, method::Put,
              method::Delete, method::Head, method::Patch];

        // If there is an override, use it.
        if let Some(matched) = self.recognize(&method::Options, path) {
            req.extensions.insert::<Router>(matched.params);
            return matched.handler.handle(req);
        }

        // Else, get all the available methods and return them.
        let mut options = vec![];

        for method in METHODS.iter() {
            self.routers.get(method).map(|router| {
                if let Some(_) = router.recognize(path).ok() {
                    options.push(method.clone());
                }
            });
        }

        let mut res = Response::with(status::Ok);
        res.headers.set(headers::Allow(options));
        Ok(res)
    }

    fn handle_trailing_slash(&self, req: &mut Request) -> IronResult<Response> {
        let mut url = req.url.clone();

        // Pull off as many trailing slashes as possible.
        while url.path.len() != 1 && url.path.last() == Some(&String::new()) {
            url.path.pop();
        }

        Err(IronError::new(TrailingSlash, (status::MovedPermanently, Redirect(url))))
    }

    fn handle_method(&self, req: &mut Request, path: &str) -> IronResult<Response> {
        let matched = match self.recognize(&req.method, path) {
            Some(matched) => matched,
            // No match.
            None => return Err(IronError::new(NoRoute, status::NotFound))
        };

        req.extensions.insert::<Router>(matched.params);
        matched.handler.handle(req)
    }
}

impl Key for Router { type Value = Params; }

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        if req.url.path.len() != 1 && Some(&String::new()) == req.url.path.last() {
            return self.handle_trailing_slash(req);
        }

        // No trailing slash
        let path = req.url.path.connect("/");

        if let method::Options = req.method {
            return self.handle_options(req, &*path);
        }

        self.handle_method(req, &*path)
    }
}

/// The error thrown by router if there is no matching route,
/// it is always accompanied by a NotFound response.
#[derive(Debug)]
pub struct NoRoute;

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("No matching route found.")
    }
}

impl Error for NoRoute {
    fn description(&self) -> &str { "No Route" }
}

/// The error thrown by router if the request had a trailing slash,
/// it is always accompanied by a redirect.
#[derive(Debug)]
pub struct TrailingSlash;

impl fmt::Display for TrailingSlash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("The request had a trailing slash.")
    }
}

impl Error for TrailingSlash {
    fn description(&self) -> &str { "Trailing Slash" }
}

