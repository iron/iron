use std::collections::HashMap;
use std::collections::hashmap::{Occupied, Vacant};
use http::method::Method;
use http::method;
use iron::{Request, Response, Handler, IronResult, Error, IronError};
use iron::status;
use typemap::Assoc;
use recognizer::Router as Recognizer;
use recognizer::{Match, Params};

/// `Router` provides an interface for creating complex routes as middleware
/// for the Iron framework.
pub struct Router {
    // The routers, specialized by method.
    routers: HashMap<Method, Recognizer<Box<Handler + Send + Sync>>>,
    error: Option<Box<Handler + Send + Sync>>
}

#[deriving(Show)]
/// The error thrown by router if there is no matching route.
pub struct NoRoute;

impl Error for NoRoute {
    fn name(&self) -> &'static str { "No Route" }
}

impl Router {
    /// `new` constructs a new, blank `Router`.
    pub fn new() -> Router { Router { routers: HashMap::new(), error: None } }
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
    /// router.route(http::method::Get, "/users/:userid/:friendid", controller);
    /// ```
    ///
    /// The controller provided to route can be any `Middleware`, which allows
    /// extreme flexibility when handling routes. For instance, you could provide
    /// a `Chain`, a `Middleware`, which contains an authorization middleware and
    /// a controller function, so that you can confirm that the request is
    /// authorized for this route before handling it.
    pub fn route<'a, H: Handler, S: Str>(&mut self, method: Method, glob: S, handler: H) -> &mut Router {
        match self.routers
            .entry(method) {
                Vacant(entry)   => entry.set(Recognizer::new()),
                Occupied(entry) => entry.into_mut()
            }
        .add(glob.as_slice(), box handler as Box<Handler + Send + Sync>);
        self
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<'a, H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Get, glob, handler)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<'a, H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Post, glob, handler)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<'a, H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Put, glob, handler)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<'a, H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Delete, glob, handler)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<'a, H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Head, glob, handler)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<'a, H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Patch, glob, handler)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<'a, H: Handler, S: Str>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Options, glob, handler)
    }

    /// Add a Handler to be used for this Router's `catch` method.
    pub fn error<H: Handler>(&mut self, handler: H) -> &mut Router {
        self.error = Some(box handler as Box<Handler + Send + Sync>);
        self
    }

    fn recognize<'a>(&'a self, method: &Method, path: &str)
                     -> Option<Match<&'a Box<Handler + Send + Sync>>> {
        self.routers.find(method).and_then(|router| router.recognize(path).ok())
    }
}

impl Assoc<Params> for Router {}

impl Handler for Router {
    fn call(&self, req: &mut Request) -> IronResult<Response> {
        let matched = match self.recognize(&req.method, req.url.path.connect("/").as_slice()) {
            Some(matched) => matched,
            // No match.
            None => return Err(NoRoute.abstract())
        };

        req.extensions.insert::<Router, Params>(matched.params);
        matched.handler.call(req)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> (Response, IronResult<()>) {
        match self.error {
            Some(ref error_handler) => error_handler.catch(req, err),
            // Error that is not caught by anything!
            None => (Response::status(status::InternalServerError), Err(err))
        }
    }
}
