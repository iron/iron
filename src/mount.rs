use http::server::request::{AbsolutePath};
use regex::Regex;

use iron::{Iron, Middleware, Request, Response, Alloy, Chain};
use iron::middleware::{Status, Continue, Unwind};

/// `Mount` is a simple mounting middleware.
///
/// `Mount` allows you to mount other instances of Iron within a larger
/// instance of Iron. Mounted instances of Iron receive a modified version
/// of the request url with the mount pattern removed from the start of the
/// url.
#[deriving(Clone)]
pub struct Mount<C> {
    route: String,
    matches: Regex,
    iron: Iron<C>
}

impl<C> Mount<C> {
    /// Creates a new instance of `Mount` mounting the given instance of Iron
    /// on the given path.
    pub fn new(route: &str, iron: Iron<C>) -> Mount<C> {
        Mount {
            route: route.to_string(),
            iron: iron,
            matches: to_regex(route)
        }
    }
}

fn to_regex(route: &str) -> Regex {
    Regex::new("^".to_string().append(route).as_slice()).unwrap()
}

impl<C: Chain> Middleware for Mount<C> {
    fn enter(&mut self,
             req: &mut Request,
             res: &mut Response,
             alloy: &mut Alloy) -> Status {
        // This method is ugly, but it is hard to make it pretty
        // because we can't both borrow path from inside of request
        // while allowing chain.dispatch to borrow it as mutable.

        match req.request_uri {
           AbsolutePath(ref path) => {
               // Short circuit if we don't match.
                if !self.matches.is_match(path.as_slice()) {
                    return Continue;
                }
           },
           // Short circuit if this is not an AbsolutePath.
           _ => { return Continue; }
        }

        // We are a match, so fire off to our child instance.
        match req.request_uri {
            AbsolutePath(ref mut path) => {
                *path = path.as_slice().slice_from(self.route.len()).to_string();
            },
            // Absolutely cannot happen because of our previous check,
            // but this is here just to be careful.
            _ => { return Continue; }
        } // Previous borrow of req ends here.

        // So we can borrow it again here.
        let _ = self.iron.chain.dispatch(req, res, Some(alloy));

        // And repair the damage here, for future middleware
        match req.request_uri {
            AbsolutePath(ref mut path) => {
                *path = self.route.clone().append(path.as_slice());
            },
            // This really, really should never happen.
            _ => { fail!("The impossible happened."); }
        }

        // We dispatched the request, so Unwind.
        Unwind
    }
}

#[macro_export]
macro_rules! mount(
    ($route:expr, $midware:expr) => {
        {
            let mut subserver: ServerT = Iron::new();
            subserver.link($midware);
            mount::Mount::new($route, subserver)
        }
    }
)

