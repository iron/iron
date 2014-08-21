use iron::{Middleware, Request, Response, Url, Status, Continue};
use typemap::Assoc;

/// Exposes the original, unmodified path to be stored in `Request::extensions`.
pub struct OriginalUrl;
impl Assoc<Url> for OriginalUrl {}

/// `Mount` is a simple mounting middleware.
///
/// `Mount` allows you to mount other middleware within a larger chain.
///
/// Mounted middleware will receive a modified version of the request
/// url with the mount pattern removed from the start of the url.
#[deriving(Clone)]
pub struct Mount<M> {
    route: String,
    matches: Vec<String>,
    middleware: M
}

impl<M: Middleware + Send> Mount<M> {
    /// Creates a new instance of `Mount` mounting the given instance of Iron
    /// on the given path.
    pub fn new(route: &str, middleware: M) -> Mount<M> {
        Mount {
            route: route.to_string(),
            middleware: middleware,
            matches: Path::new(route)
                        .str_components()
                        .map(|s| s.unwrap().to_string())
                        .collect()
        }
    }
}

impl<M: Middleware + Send> Middleware for Mount<M> {
    fn enter(&mut self,
             req: &mut Request,
             res: &mut Response) -> Status {
        // Check for a route match.
        if !req.url.path.as_slice().starts_with(self.matches.as_slice()) {
            return Continue;
        }

        // We have a match, so fire off the child middleware.
        // If another mount middleware hasn't already, insert the unmodified url
        // into the extensions as the "original url".
        let is_outer_mount = !req.extensions.contains::<OriginalUrl, Url>();
        if is_outer_mount {
            req.extensions.insert::<OriginalUrl, Url>(req.url.clone());
        }

        // Remove the prefix from the request's path before passing it to the mounted middleware.
        // Preserve the rust-url invariant that the path list is non-empty ("" corresponds to /).
        req.url.path = match req.url.path.as_slice().slice_from(self.matches.len()) {
            [] => vec!["".to_string()],
            list => list.to_vec()
        };

        let terminator = self.middleware.enter(req, res);
        let _ = self.middleware.exit(req, res);

        // Reverse the URL munging, for future middleware.
        req.url = match req.extensions.find::<OriginalUrl, Url>() {
            Some(original) => original.clone(),
            None => fail!("OriginalUrl unexpectedly removed from req.extensions.")
        };

        // If this middleware is the outermost mount middleware,
        // remove the original url from the extensions map to prevent leakage.
        if is_outer_mount {
            req.extensions.remove::<OriginalUrl, Url>();
        }

        // We dispatched the request, so return the terminator.
        terminator
    }
}

