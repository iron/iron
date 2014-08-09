use url::Url;
use iron::{Middleware, Request, Response, Status, Continue};
use collections::slice::ImmutableEqVector;

/// Exposes the original, unmodified path to be stored in an Alloy.
pub struct OriginalUrl(pub Url);

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
        // Unwrapping url.path() is safe because the request is HTTP.
        // XXX: If a url_path() -> &[String] method is added to Request, use that.
        macro_rules! url_path(
            ($req:ident) => (
                $req.url.path().unwrap()
            )
        )

        // Check for a route match.
        if !url_path!(req).starts_with(self.matches.as_slice()) {
            return Continue;
        }

        // We are a match, so fire off to our child instance.
        // Insert the unmodified path into the alloy.
        match req.alloy.find::<OriginalUrl>() {
            Some(_) => (),
            None => req.alloy.insert(OriginalUrl(req.url.clone()))
        }

        // Remove the prefix from the request's path before passing it to the mounted middleware.
        // This unwrap is safe because url.path() is guaranteed to not be None.
        *req.url.path_mut().unwrap() = url_path!(req).slice_from(self.matches.len()).to_vec();

        let terminator = self.middleware.enter(req, res);
        let _ = self.middleware.exit(req, res);

        // Reverse the URL munging, for future middleware.
        let &OriginalUrl(ref original) = req.alloy.find::<OriginalUrl>().unwrap();
        req.url = original.clone();

        // We dispatched the request, so return the terminator.
        terminator
    }
}

