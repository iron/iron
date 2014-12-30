use std::error::Error;
use iron::{Handler, Response, Request, IronResult, IronError, Url};
use iron::typemap::Assoc;
use sequence_trie::SequenceTrie;

/// Exposes the original, unmodified path to be stored in `Request::extensions`.
#[deriving(Copy)]
pub struct OriginalUrl;
impl Assoc<Url> for OriginalUrl {}

/// `Mount` is a simple mounting middleware.
///
/// Mounting allows you to install a handler on a route and have it receive requests as if they
/// are relative to that route. For example, a handler mounted on `/foo/` will receive
/// requests like `/foo/bar` as if they are just `/bar`. Iron's mounting middleware allows
/// you to specify multiple mountings using one middleware instance. Requests that pass through
/// the mounting middleware are passed along to the mounted handler that best matches the request's
/// path. `Request::url` is modified so that requests appear to be relative to the mounted handler's route.
///
/// Mounted handlers may also access the *original* URL by requesting the `OriginalUrl` key
/// from `Request::extensions`.
pub struct Mount {
    inner: SequenceTrie<String, Match>
}

struct Match {
    handler: Box<Handler + Send + Sync>,
    length: uint
}

/// The error returned by `Mount` when a request doesn't match any mounted handlers.
#[deriving(Show)]
pub struct NoMatch;

impl Error for NoMatch {
    fn description(&self) -> &'static str { "No Match" }
}

impl Mount {
    /// Creates a new instance of `Mount`.
    pub fn new() -> Mount {
        Mount {
            inner: SequenceTrie::new()
        }
    }

    /// Mounts a given `Handler` onto a route.
    ///
    /// This method may be called multiple times with different routes.
    /// For a given request, the *most specific* handler will be selected.
    ///
    /// Existing handlers on the same route will be overwritten.
    pub fn mount<H: Handler>(&mut self, route: &str, handler: H) -> &mut Mount {
        // Parse the route into a list of strings. The unwrap is safe because strs are UTF-8.
        let key: Vec<String> = Path::new(route).str_components().map(|s| s.unwrap().to_string()).collect();

        // Insert a match struct into the trie.
        self.inner.insert(key.as_slice(), Match {
            handler: box handler as Box<Handler + Send + Sync>,
            length: key.len()
        });
        self
    }
}

impl Handler for Mount {
    fn call(&self, req: &mut Request) -> IronResult<Response> {
        // Find the matching handler.
        let matched = {
            // Extract the request path.
            let path = req.url.path.as_slice();

            // If present, remove the trailing empty string (which represents a trailing slash).
            // If it isn't removed the path will never match anything, because
            // Path::str_components ignores trailing slashes and will never create routes
            // ending in "".
            let key = match path.last() {
                Some(s) if s.is_empty() => path.slice_to(path.len() - 1),
                _ => path
            };

            // Search the Trie for the nearest most specific match.
            match self.inner.get_ancestor(key) {
                Some(matched) => matched,
                None => return Err(box NoMatch as IronError)
            }
        };

        // We have a match, so fire off the child.
        // If another mount middleware hasn't already, insert the unmodified url
        // into the extensions as the "original url".
        let is_outer_mount = !req.extensions.contains::<OriginalUrl, Url>();
        if is_outer_mount {
            req.extensions.insert::<OriginalUrl, Url>(req.url.clone());
        }

        // Remove the prefix from the request's path before passing it to the mounted handler.
        // If the prefix is entirely removed and no trailing slash was present, the new path
        // will be the empty list. For the purposes of redirection, conveying that the path
        // did not include a trailing slash is more important than providing a non-empty list.
        req.url.path = req.url.path.as_slice().slice_from(matched.length).to_vec();

        let res = matched.handler.call(req);

        // Reverse the URL munging, for future middleware.
        req.url = match req.extensions.get::<OriginalUrl, Url>() {
            Some(original) => original.clone(),
            None => panic!("OriginalUrl unexpectedly removed from req.extensions.")
        };

        // If this mount middleware is the outermost mount middleware,
        // remove the original url from the extensions map to prevent leakage.
        if is_outer_mount {
            req.extensions.remove::<OriginalUrl, Url>();
        }

        res
    }
}

