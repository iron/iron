use regex::Regex;

use iron::{Middleware, Request, Response, Alloy, Chain, Status, Continue};
use iron::mixin::GetUrl;

/// Exposes the original, unmodified path to be stored in an Alloy.
pub struct OriginalUrl(pub String);

/// `Mount` is a simple mounting middleware.
///
/// `Mount` allows you to mount other middleware within a larger chain.
///
/// Mounted middleware will receive a modified version of the request
/// url with the mount pattern removed from the start of the url.
pub struct Mount {
    route: String,
    matches: Regex,
    middleware: Box<Middleware + Send>
}

impl Clone for Mount {
    fn clone(&self) -> Mount {
        Mount {
            route: self.route.clone(),
            matches: self.matches.clone(),
            middleware: self.middleware.clone_box()
        }
    }
}

impl Mount {
    /// Creates a new instance of `Mount` mounting the given instance of Iron
    /// on the given path.
    pub fn new<M: Middleware + Send>(route: &str, middleware: M) -> Mount {
        Mount {
            route: route.to_string(),
            middleware: box middleware,
            matches: to_regex(route)
        }
    }
}

fn to_regex(route: &str) -> Regex {
    Regex::new("^".to_string().append(route).as_slice()).unwrap()
}

impl Middleware for Mount {
    fn enter(&mut self,
             req: &mut Request,
             res: &mut Response,
             alloy: &mut Alloy) -> Status {
        // This method is ugly, but it is hard to make it pretty
        // because we can't both borrow path from inside of request
        // while allowing chain.dispatch to borrow it as mutable.

        match req.url() {
           Some(path) => {
               // Short circuit if we don't match.
                if !self.matches.is_match(path.as_slice()) {
                    return Continue;
                }
           },
           // Short circuit if this is not an AbsolutePath.
           None => { return Continue; }
        }

        // We are a match, so fire off to our child instance.
        match req.url_mut() {
            Some(path) => {
                // Insert the unmodified path into the alloy.
                match alloy.find::<OriginalUrl>() {
                    Some(_) => (),
                    None => alloy.insert(OriginalUrl(path.clone()))
                }
                *path = path.as_slice().slice_from(self.route.len()).to_string();
            },
            // Absolutely cannot happen because of our previous check,
            // but this is here just to be careful.
            None => { return Continue; }
        } // Previous borrow of req ends here.

        // So we can borrow it again here.
        let terminator = self.middleware.enter(req, res, alloy);
        let _ = self.middleware.exit(req, res, alloy);

        // And repair the damage here, for future middleware
        match req.url_mut() {
            Some(path) => {
                *path = self.route.clone().append(path.as_slice());
            },
            // This really, really should never happen.
            None => { fail!("The impossible happened."); }
        }

        // We dispatched the request, so return the terminator.
        terminator
    }
}

