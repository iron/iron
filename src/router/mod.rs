use std::fmt::Show;
use std::collections::HashMap;
use http::method::Method;
use http::method;
use iron::{Middleware, Request, Response, Status, Continue, Unwind, Error};
use Recognizer = recognizer::Router;
use recognizer::Match;

/// `Router` provides an interface for creating complex routes as middleware
/// for the Iron framework.
#[deriving(Clone)]
pub struct Router {
    // The routers, specialized by method.
    routers: HashMap<Method, Recognizer<Box<Middleware + Send>>>
}

impl Router {
    /// `new` constructs a new, blank `Router`.
    pub fn new() -> Router { Router { routers: HashMap::new() } }
    /// Add a new route to a `Router`, matching both a method and glob pattern.
    ///
    /// `route` supports glob patterns: `*` for a single wildcard segment and
    /// `:param` for matching storing that segment of the request url in the `Params`
    /// object, which is stored on the request Alloy.
    ///
    /// For instance, to route `Get` requests on any route matching
    /// `/users/:userid/:friend` and store `userid` and `friend` in
    /// the exposed Params object:
    ///
    /// ```ignore
    /// router.route(http::method::Get, "/users/:userid/:friendid", controller);
    /// ```
    ///
    /// The controller provided to route can be any `Middleware`, which allows
    /// extreme flexibility when handling routes. For instance, you could provide
    /// a `Chain`, a `Middleware`, which contains an authorization middleware and
    /// a controller function, so that you can confirm that the request is
    /// authorized for this route before handling it.
    pub fn route<'a, M: Middleware + Send, S: Str>(&mut self, method: Method, glob: S, handler: M) -> &mut Router {
        self.routers
            .find_or_insert_with(method, |_| Recognizer::new())
            .add(glob.as_slice(), box handler as Box<Middleware + Send>);
        self
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<'a, M: Middleware + Send, S: Str>(&mut self, glob: S, handler: M) -> &mut Router {
        self.route(method::Get, glob, handler)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<'a, M: Middleware + Send, S: Str>(&mut self, glob: S, handler: M) -> &mut Router {
        self.route(method::Post, glob, handler)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<'a, M: Middleware + Send, S: Str>(&mut self, glob: S, handler: M) -> &mut Router {
        self.route(method::Put, glob, handler)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<'a, M: Middleware + Send, S: Str>(&mut self, glob: S, handler: M) -> &mut Router {
        self.route(method::Delete, glob, handler)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<'a, M: Middleware + Send, S: Str>(&mut self, glob: S, handler: M) -> &mut Router {
        self.route(method::Head, glob, handler)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<'a, M: Middleware + Send, S: Str>(&mut self, glob: S, handler: M) -> &mut Router {
        self.route(method::Patch, glob, handler)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<'a, M: Middleware + Send, S: Str>(&mut self, glob: S, handler: M) -> &mut Router {
        self.route(method::Options, glob, handler)
    }

    fn recognize<'a>(&'a self, method: &Method, path: &str)
                     -> Option<Match<&'a Box<Middleware + Send>>> {
        self.routers.find(method).and_then(|router| router.recognize(path).ok())
    }
}

impl Middleware for Router {
    fn enter(&mut self, req: &mut Request, res: &mut Response) -> Status {
        let matched = match self.recognize(&req.method, req.url.path.connect("/").as_slice()) {
            Some(matched) => matched,
            // No match.
            None => return Continue
        };

        req.extensions.insert(matched.params);
        let mut handler = matched.handler.clone_box();
        let mut handler_status = handler.enter(req, res);

        match handler_status {
            Error(ref mut e) => {
                let error: &mut Show = *e;
                let _ = handler.on_error(req, res, error);
            },
            _ => {
                let _ = handler.exit(req, res);
            }
        };

        return match handler_status {
            Continue => Unwind,
            otherwise => otherwise
        }
    }
}

