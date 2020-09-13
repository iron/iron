use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use iron::modifiers::Redirect;
use iron::typemap::Key;
use iron::{headers, method, Method, StatusCode};
use iron::{Handler, IronError, IronResult, Request, Response};

use recognizer::Router as Recognizer;
use recognizer::{Match, Params};

pub struct RouterInner {
    // The routers, specialized by method.
    pub routers: HashMap<method::Method, Recognizer<Box<dyn Handler>>>,
    // Routes that accept any method.
    pub wildcard: Recognizer<Box<dyn Handler>>,
    // Used in URL generation.
    pub route_ids: HashMap<String, String>,
}

/// `Router` provides an interface for creating complex routes as middleware
/// for the Iron framework.
pub struct Router {
    inner: Arc<RouterInner>,
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
                routers: HashMap::new(),
                wildcard: Recognizer::new(),
                route_ids: HashMap::new(),
            }),
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
    pub fn route<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        method: method::Method,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.mut_inner()
            .routers
            .entry(method)
            .or_insert(Recognizer::new())
            .add(glob.as_ref(), Box::new(handler));
        self.route_id(route_id.as_ref(), glob.as_ref());
        self
    }

    fn route_id(&mut self, id: &str, glob: &str) {
        let inner = self.mut_inner();
        let ref mut route_ids = inner.route_ids;

        match route_ids.get(id) {
            Some(other_glob) if glob != other_glob => panic!("Duplicate route_id: {}", id),
            _ => (),
        };

        route_ids.insert(id.to_owned(), glob.to_owned());
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.route(Method::GET, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.route(Method::POST, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Put` method.
    pub fn put<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.route(Method::PUT, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Delete` method.
    pub fn delete<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.route(Method::DELETE, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Head` method.
    pub fn head<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.route(Method::HEAD, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Patch` method.
    pub fn patch<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.route(Method::PATCH, glob, handler, route_id)
    }

    /// Like route, but specialized to the `Options` method.
    pub fn options<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.route(Method::OPTIONS, glob, handler, route_id)
    }

    /// Route will match any method, including gibberish.
    /// In case of ambiguity, handlers specific to methods will be preferred.
    pub fn any<S: AsRef<str>, H: Handler, I: AsRef<str>>(
        &mut self,
        glob: S,
        handler: H,
        route_id: I,
    ) -> &mut Router {
        self.mut_inner()
            .wildcard
            .add(glob.as_ref(), Box::new(handler));
        self.route_id(route_id.as_ref(), glob.as_ref());
        self
    }

    fn recognize(&self, method: &method::Method, path: &str) -> Option<Match<&Box<dyn Handler>>> {
        self.inner
            .routers
            .get(method)
            .and_then(|router| router.recognize(path).ok())
            .or(self.inner.wildcard.recognize(path).ok())
    }

    fn handle_options(&self, path: &str) -> Response {
        static METHODS: &'static [method::Method] = &[
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::PATCH,
        ];

        // Get all the available methods and return them.
        let mut options = vec![];

        for method in METHODS.iter() {
            self.inner.routers.get(method).map(|router| {
                if let Some(_) = router.recognize(path).ok() {
                    options.push(method.clone());
                }
            });
        }
        // If GET is there, HEAD is also there.
        if options.contains(&Method::GET) && !options.contains(&Method::HEAD) {
            options.push(Method::HEAD);
        }

        let mut res = Response::with(StatusCode::OK);
        for option in options {
            res.headers
                .append(headers::ALLOW, option.as_str().parse().unwrap());
        }
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

        self.recognize(&req.method, &path).and(Some(IronError::new(
            TrailingSlash,
            (StatusCode::MOVED_PERMANENTLY, Redirect(url)),
        )))
    }

    fn handle_method(&self, req: &mut Request, path: &str) -> Option<IronResult<Response>> {
        if let Some(matched) = self.recognize(&req.method, path) {
            req.extensions.insert::<Router>(matched.params);
            req.extensions.insert::<RouterInner>(self.inner.clone());
            Some(matched.handler.handle(req))
        } else {
            self.redirect_slash(req)
                .and_then(|redirect| Some(Err(redirect)))
        }
    }
}

impl Key for Router {
    type Value = Params;
}

impl Key for RouterInner {
    type Value = Arc<RouterInner>;
}

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path().join("/");

        self.handle_method(req, &path)
            .unwrap_or_else(|| match req.method {
                Method::OPTIONS => Ok(self.handle_options(&path)),
                // For HEAD, fall back to GET. Hyper ensures no response body is written.
                Method::HEAD => {
                    req.method = Method::GET;
                    self.handle_method(req, &path)
                        .unwrap_or(Err(IronError::new(NoRoute, StatusCode::NOT_FOUND)))
                }
                _ => Err(IronError::new(NoRoute, StatusCode::NOT_FOUND)),
            })
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

impl Error for NoRoute {}

/// The error thrown by router if a request was redirected
/// by adding or removing a trailing slash.
#[derive(Debug, PartialEq, Eq)]
pub struct TrailingSlash;

impl fmt::Display for TrailingSlash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("The request had a trailing slash.")
    }
}

impl Error for TrailingSlash {}

#[cfg(test)]
mod test {
    use super::Router;
    use iron::{headers, method, Method, Request, Response, StatusCode};

    #[test]
    fn test_handle_options_post() {
        let mut router = Router::new();
        router.post(
            "/",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );
        let resp = router.handle_options("/");
        let headers: Vec<method::Method> = resp
            .headers
            .get_all(headers::ALLOW)
            .into_iter()
            .map(|s| s.to_str().unwrap().parse().unwrap())
            .collect();
        let expected = vec![Method::POST];
        assert_eq!(expected, headers);
    }

    #[test]
    fn test_handle_options_get_head() {
        let mut router = Router::new();
        router.get(
            "/",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );
        let resp = router.handle_options("/");
        let headers: Vec<method::Method> = resp
            .headers
            .get_all(headers::ALLOW)
            .into_iter()
            .map(|s| s.to_str().unwrap().parse().unwrap())
            .collect();
        let expected = vec![method::Method::GET, method::Method::HEAD];
        assert_eq!(expected, headers);
    }

    #[test]
    fn test_handle_any_ok() {
        let mut router = Router::new();
        router.post(
            "/post",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );
        router.any(
            "/post",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );
        router.put(
            "/post",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );
        router.any(
            "/get",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "any",
        );

        assert!(router.recognize(&Method::GET, "/post").is_some());
        assert!(router.recognize(&Method::GET, "/get").is_some());
    }

    #[test]
    fn test_request() {
        let mut router = Router::new();
        router.post(
            "/post",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );
        router.get(
            "/post",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );

        assert!(router.recognize(&Method::POST, "/post").is_some());
        assert!(router.recognize(&Method::GET, "/post").is_some());
        assert!(router.recognize(&Method::PUT, "/post").is_none());
        assert!(router.recognize(&Method::GET, "/post/").is_none());
    }

    #[test]
    fn test_not_found() {
        let mut router = Router::new();
        router.put(
            "/put",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "",
        );
        assert!(router.recognize(&Method::PATCH, "/patch").is_none());
    }

    #[test]
    #[should_panic]
    fn test_same_route_id() {
        let mut router = Router::new();
        router.put(
            "/put",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "my_route_id",
        );
        router.get(
            "/get",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "my_route_id",
        );
    }

    #[test]
    fn test_wildcard_regression() {
        let mut router = Router::new();
        router.options(
            "*",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "id1",
        );
        router.put(
            "/upload/*filename",
            |_: &mut Request| Ok(Response::with((StatusCode::OK, ""))),
            "id2",
        );
        assert!(router.recognize(&Method::OPTIONS, "/foo").is_some());
        assert!(router.recognize(&Method::PUT, "/foo").is_none());
        assert!(router.recognize(&Method::PUT, "/upload/foo").is_some());
    }
}
