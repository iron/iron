use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use iron::{Request, Response, Handler, IronResult, IronError};
use iron::{status, method, headers};
use iron::typemap::Key;
use iron::modifiers::Redirect;

use recognizer::Router as Recognizer;
use recognizer::{Match, Params};

pub struct RouterInner {
    // The routers, specialized by method.
    pub routers: Recognizer<HashMap<method::Method, Arc<Handler>>>,
    // Routes that accept any method.
    pub wildcard: Recognizer<Arc<Handler>>,
    // Used in URL generation.
    pub route_ids: HashMap<String, String>
}

/// `Router` provides an interface for creating complex routes as middleware
/// for the Iron framework.
pub struct Router {
    inner: Arc<RouterInner>
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
            inner: Arc::new(RouterInner {
                routers: Recognizer::new(),
                wildcard: Recognizer::new(),
                route_ids: HashMap::new()
            })
        }
    }

    fn mut_inner(&mut self) -> &mut RouterInner {
        Arc::get_mut(&mut self.inner).expect("Cannot modify router at this point.")
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
    /// router.route(method::Get, "/users/:userid/:friendid", controller, "user_friend");
    /// ```
    ///
    /// `route_id` is a unique name for your route, and is used when generating an URL with
    /// `url_for`.
    ///
    /// The controller provided to route can be any `Handler`, which allows
    /// extreme flexibility when handling routes. For instance, you could provide
    /// a `Chain`, a `Handler`, which contains an authorization middleware and
    /// a controller function, so that you can confirm that the request is
    /// authorized for this route before handling it.
    pub fn route<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, method: method::Method, glob: S, handler: H, route_id: I) -> &mut Router {

        let mut hash: HashMap<method::Method, Arc<Handler>>;

        if let Some(s) = self.mut_inner().routers.recognize(glob.as_ref()).ok() {
            hash = s.handler.clone();
        } else {
            hash = HashMap::new();
        }

        hash.insert(method, Arc::new(handler));
        self.mut_inner().routers.add(glob.as_ref(), hash);
        self.route_id(route_id.as_ref(), glob.as_ref());
        self
    }

    fn route_id(&mut self, id: &str, glob: &str) {
        let mut mut_inner = self.mut_inner();
        let ref mut route_ids = mut_inner.route_ids;

        match route_ids.get(id) {
            Some(other_glob) if glob != other_glob => panic!("Duplicate route_id: {}", id),
            _ => ()
        };

        route_ids.insert(id.to_owned(), glob.to_owned());
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.route(method::Get, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.route(method::Post, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.route(method::Put, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.route(method::Delete, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.route(method::Head, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.route(method::Patch, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.route(method::Options, glob, handler, route_id)
    }

    /// Route will match any method, including gibberish.
    /// In case of ambiguity, handlers specific to methods will be preferred.
    pub fn any<S: AsRef<str>, H: Handler, I: AsRef<str>>(&mut self, glob: S, handler: H, route_id: I) -> &mut Router {
        self.mut_inner().wildcard.add(glob.as_ref(), Arc::new(handler));
        self.route_id(route_id.as_ref(), glob.as_ref());
        self
    }

    fn recognize(&self, method: &method::Method, path: &str)
                      -> Result<Match<&Arc<Handler>>, RouterError> {
        match self.inner.routers.recognize(path)
            .map(|s|
                match s.handler.get(method) {
                    Some(h) => Ok(Match::new(h, s.params)),
                    None => self.inner.wildcard.recognize(path).ok().map_or(Err(RouterError::MethodNotAllowed), |s| Ok(s))
                }
        ).map_err(|_| self.inner.wildcard.recognize(path).ok().map_or(Err(RouterError::NotFound), |s| Ok(s))) {
            Ok(s) => s,
            Err(e) => e
        }
    }

    fn handle_options(&self, path: &str) -> Response {
        static METHODS: &'static [method::Method] =
            &[method::Get, method::Post, method::Put,
              method::Delete, method::Head, method::Patch];

        // Get all the available methods and return them.
        let mut options = vec![];

        for method in METHODS.iter() {
            if let Ok(s) = self.inner.routers.recognize(path) {
                if let Some(_) = s.handler.get(method) {
                    options.push(method.clone());
                }
            }
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
    fn redirect_slash(&self, req: &Request) -> Option<IronError> {
        let mut url = req.url.clone();
        let mut path = url.path().join("/");

        if let Some(last_char) = path.chars().last() {
            {
                let mut path_segments = url.as_mut().path_segments_mut().unwrap();
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
        }

        self.recognize(&req.method, &path).ok().and(
            Some(IronError::new(RouterError::TrailingSlash,
                                (status::MovedPermanently, Redirect(url))))
        )
    }

    fn handle_method(&self, req: &mut Request, path: &str) -> IronResult<Response> {
        match self.recognize(&req.method, path) {
            Ok(matched) => {
                req.extensions.insert::<Router>(matched.params);
                req.extensions.insert::<RouterInner>(self.inner.clone());
                matched.handler.handle(req)
            },
            Err(RouterError::MethodNotAllowed) => {
                Err(IronError::new(RouterError::MethodNotAllowed, status::MethodNotAllowed))
            },
            Err(_) => {
                if let Some(err) = self.redirect_slash(req) {
                    return Err(err)
                };

                match req.method {
                    method::Options => Ok(self.handle_options(path)),
                    // For HEAD, fall back to GET. Hyper ensures no response body is written.
                    method::Head => {
                        req.method = method::Get;
                        self.handle_method(req, path)
                    },
                    _ => Err(IronError::new(RouterError::NotFound, status::NotFound))
                }
            }
        }
    }
}

impl Key for Router { type Value = Params; }

impl Key for RouterInner { type Value = Arc<RouterInner>; }

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path().join("/");
        self.handle_method(req, &path)
    }
}

/// A set of errors that can occur parsing HTTP streams.
#[derive(Debug, PartialEq)]
pub enum RouterError {
    /// The error thrown by router if there is no matching method in existing route.
    MethodNotAllowed,
    /// The error thrown by router if there is no matching route.
    NotFound,
    /// The error thrown by router if a request was redirected by adding or removing a trailing slash.
    TrailingSlash
}


impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description())
    }
}

impl Error for RouterError {
    fn description(&self) -> &str {
        match *self {
            RouterError::MethodNotAllowed => "Method Not Allowed",
            RouterError::NotFound => "No matching route found.",
            RouterError::TrailingSlash => "The request had a trailing slash."
        }
    }
}

/// The error thrown by router if there is no matching route,
/// it is always accompanied by a NotFound response.
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub struct TrailingSlash;

impl fmt::Display for TrailingSlash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("The request had a trailing slash.")
    }
}

impl Error for TrailingSlash {
    fn description(&self) -> &str { "Trailing Slash" }
}

#[cfg(test)]
mod test {
    use super::{Router, RouterError};
    use iron::{headers, method, status, Request, Response};

    #[test]
    fn test_handle_options_post() {
        let mut router = Router::new();
        router.post("/", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");
        let resp = router.handle_options("/");
        let headers = resp.headers.get::<headers::Allow>().unwrap();
        let expected = headers::Allow(vec![method::Method::Post]);
        assert_eq!(&expected, headers);
    }

    #[test]
    fn test_handle_options_get_head() {
        let mut router = Router::new();
        router.get("/", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");
        let resp = router.handle_options("/");
        let headers = resp.headers.get::<headers::Allow>().unwrap();
        let expected = headers::Allow(vec![method::Method::Get, method::Method::Head]);
        assert_eq!(&expected, headers);
    }

    #[test]
    fn test_not_allowed_method() {
        let mut router = Router::new();

        router.post("/post", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");

        router.get("/post/", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "another_route");

        match router.recognize(&method::Get, "/post") {
            Ok(_) => {
                panic!();
            },
            Err(e) => {
                assert_eq!(RouterError::MethodNotAllowed, e);
            }
        }
    }

    #[test]
    fn test_handle_any_ok() {
        let mut router = Router::new();
        router.post("/post", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");
        router.any("/post", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");
        router.put("/post", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");
        router.any("/get", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "any");

        assert!(router.recognize(&method::Get, "/post").is_ok());
        assert!(router.recognize(&method::Get, "/get").is_ok());
    }

    #[test]
    fn test_request() {
        let mut router = Router::new();
        router.post("/post", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");
        router.get("/post", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");

        assert!(router.recognize(&method::Post, "/post").is_ok());
        assert!(router.recognize(&method::Get, "/post").is_ok());
        assert!(router.recognize(&method::Put, "/post").is_err());
        assert!(router.recognize(&method::Get, "/post/").is_err());
    }

    #[test]
    fn test_not_found() {
        let mut router = Router::new();
        router.put("/put", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "");
        match router.recognize(&method::Patch, "/patch") {
            Ok(_) => {
                panic!();
            },
            Err(e) => {
                assert_eq!(RouterError::NotFound, e);
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_same_route_id() {
        let mut router = Router::new();
        router.put("/put", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "my_route_id");
        router.get("/get", |_: &mut Request| {
            Ok(Response::with((status::Ok, "")))
        }, "my_route_id");
    }
}
