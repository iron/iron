use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use iron::{Request, Response, Handler, IronResult, IronError};
use iron::{status, method, headers};
use iron::typemap::Key;
use iron::modifiers::Redirect;
use iron::Url;

use recognizer::Router as Recognizer;
use recognizer::{Match, Params};

/// `Router` provides an interface for creating complex routes as middleware
/// for the Iron framework.
pub struct Router {
    // The routers, specialized by method.
    routers: HashMap<method::Method, Recognizer<Box<Handler>>>,
    // Routes that accept any method.
    wildcard: Recognizer<Box<Handler>>
}

impl Router {
    /// Construct a new, empty `Router`.
    ///
    /// ```
    /// # use router::Router;
    /// let router = Router::new();
    /// ```
    pub fn new() -> Router {
        Router {
            routers: HashMap::new(),
            wildcard: Recognizer::new()
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
    where H: Handler, S: AsRef<str> {
        self.routers.entry(method).or_insert(Recognizer::new())
                    .add(glob.as_ref(), Box::new(handler));
        self
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Get, glob, handler)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Post, glob, handler)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Put, glob, handler)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Delete, glob, handler)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Head, glob, handler)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Patch, glob, handler)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.route(method::Options, glob, handler)
    }

    /// Route will match any method, including gibberish.
    /// In case of ambiguity, handlers specific to methods will be preferred.
    pub fn any<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &mut Router {
        self.wildcard.add(glob.as_ref(), Box::new(handler));
        self
    }

    fn recognize(&self, method: &method::Method, path: &str)
                     -> Option<Match<&Box<Handler>>> {
        self.routers.get(method).and_then(|router| router.recognize(path).ok())
            .or(self.wildcard.recognize(path).ok())
    }

    fn handle_options(&self, path: &str) -> Response {
        static METHODS: &'static [method::Method] =
            &[method::Get, method::Post, method::Post, method::Put,
              method::Delete, method::Head, method::Patch];

        // Get all the available methods and return them.
        let mut options = vec![];

        for method in METHODS.iter() {
            self.routers.get(method).map(|router| {
                if let Some(_) = router.recognize(path).ok() {
                    options.push(method.clone());
                }
            });
        }
        // If GET is there, HEAD is also there.
        if options.contains(&method::Get) && !options.contains(&method::Head) {
            options.push(method::Head);
        }

        let mut res = Response::with(status::Ok);
        res.headers.set(headers::Allow(options));
        res
    }

    // Tests for a match by adding or removing a trailing slash.
    fn redirect_slash(&self, req : &Request) -> Option<IronError> {
        let mut url = req.url.clone();
        let mut path = url.path().join("/");

        if let Some(last_char) = path.chars().last() {
            // Unwrap generic URL to get access to its path components.
            let mut generic_url = url.into_generic_url();
            {
                let mut path_segments = generic_url.path_segments_mut().unwrap();
                if last_char == '/' {
                    // We didn't recognize anything without a trailing slash; try again with one appended.
                    path.pop();
                    path_segments.pop();
                } else {
                    // We didn't recognize anything with a trailing slash; try again without it.
                    path.push('/');
                    path_segments.push("");
                }
            }
            url = Url::from_generic_url(generic_url).unwrap();
        }

        self.recognize(&req.method, &path).and(
            Some(IronError::new(TrailingSlash,
                                (status::MovedPermanently, Redirect(url))))
        )
    }

    fn handle_method(&self, req: &mut Request, path: &str) -> Option<IronResult<Response>> {
        if let Some(matched) = self.recognize(&req.method, path) {
            req.extensions.insert::<Router>(matched.params);
            Some(matched.handler.handle(req))
        } else { self.redirect_slash(req).and_then(|redirect| Some(Err(redirect))) }
    }
}

impl Key for Router { type Value = Params; }

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path().join("/");

        self.handle_method(req, &path).unwrap_or_else(||
            match req.method {
                method::Options => Ok(self.handle_options(&path)),
                // For HEAD, fall back to GET. Hyper ensures no response body is written.
                method::Head => {
                    req.method = method::Get;
                    self.handle_method(req, &path).unwrap_or(
                        Err(IronError::new(NoRoute, status::NotFound))
                    )
                }
                _ => Err(IronError::new(NoRoute, status::NotFound))
            }
        )
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

/// The error thrown by router if a request was redirected
/// by adding or removing a trailing slash.
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
