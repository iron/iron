use regex::Regex;

use iron::{Middleware, Request, Response, Alloy, Status, Continue};

/// Exposes the original, unmodified path to be stored in an Alloy.
pub struct OriginalUrl(pub String);

/// `Mount` is a simple mounting middleware.
///
/// `Mount` allows you to mount other middleware within a larger chain.
///
/// Mounted middleware will receive a modified version of the request
/// url with the mount pattern removed from the start of the url.
#[deriving(Clone)]
pub struct Mount<M> {
    route: String,
    matches: Regex,
    middleware: M
}

impl<M: Middleware + Send> Mount<M> {
    /// Creates a new instance of `Mount` mounting the given instance of Iron
    /// on the given path.
    pub fn new(route: &str, middleware: M) -> Mount<M> {
        Mount {
            route: route.to_string(),
            middleware: middleware,
            matches: to_regex(route)
        }
    }
}

fn to_regex(route: &str) -> Regex {
    Regex::new("^".to_string().append(route).as_slice()).unwrap()
}

impl<M: Middleware + Send> Middleware for Mount<M> {
    fn enter(&mut self,
             req: &mut Request,
             res: &mut Response,
             alloy: &mut Alloy) -> Status {
        if !self.matches.is_match(req.url.as_slice()) {
            return Continue;
        }

        // We are a match, so fire off to our child instance.
        // Insert the unmodified path into the alloy.
        match alloy.find::<OriginalUrl>() {
            Some(_) => (),
            None => alloy.insert(OriginalUrl(req.url.clone()))
        }
        req.url = req.url.as_slice().slice_from(self.route.len()).to_string();

        let terminator = self.middleware.enter(req, res, alloy);
        let _ = self.middleware.exit(req, res, alloy);

        // And repair the damage here, for future middleware
        let &OriginalUrl(ref original) = alloy.find::<OriginalUrl>().unwrap();
        req.url = original.clone();

        // We dispatched the request, so return the terminator.
        terminator
    }
}

