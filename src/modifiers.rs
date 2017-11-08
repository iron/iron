//! This module defines a series of convenience modifiers for changing
//! Responses.
//!
//! Modifiers can be used to edit `Response`s through the owning method `set`
//! or the mutating `set_mut`, both of which are defined in the `Set` trait.
//!
//! For Iron, the `Modifier` interface offers extensible and ergonomic response
//! creation while avoiding the introduction of many highly specific `Response`
//! constructors.
//!
//! The simplest case of a modifier is probably the one used to change the
//! return status code:
//!
//! ```
//! # use iron::prelude::*;
//! # use iron::status;
//! let r = Response::with(status::NotFound);
//! assert_eq!(r.status.unwrap().to_u16(), 404);
//! ```
//!
//! You can also pass in a tuple of modifiers, they will all be applied. Here's
//! an example of a modifier 2-tuple that will change the status code and the
//! body message:
//!
//! ```
//! # use iron::prelude::*;
//! # use iron::status;
//! Response::with((status::ImATeapot, "I am a tea pot!"));
//! ```
//!
//! There is also a `Redirect` modifier:
//!
//! ```
//! # use iron::prelude::*;
//! # use iron::status;
//! # use iron::modifiers;
//! # use iron::Url;
//! let url = Url::parse("http://doc.rust-lang.org").unwrap();
//! Response::with((status::Found, modifiers::Redirect(url)));
//! ```
//!
//! The modifiers are applied depending on their type. Currently the easiest
//! way to see how different types are used as modifiers, take a look at [the
//! source code](https://github.com/iron/iron/blob/master/src/modifiers.rs).
//!
//! For more information about the modifier system, see
//! [rust-modifier](https://github.com/reem/rust-modifier).

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use modifier::Modifier;

use hyper::mime::{Mime, SubLevel, TopLevel};

use {status, headers, Request, Response, Set, Url};

use mime_guess::guess_mime_type_opt;
use response::{WriteBody, BodyReader};


impl Modifier<Response> for Mime {
    #[inline]
    fn modify(self, res: &mut Response) {
        res.headers.set(headers::ContentType(self))
    }
}

impl Modifier<Response> for Box<WriteBody> {
    #[inline]
    fn modify(self, res: &mut Response) {
        res.body = Some(self);
    }
}

impl <R: io::Read + Send + 'static> Modifier<Response> for BodyReader<R> {
    #[inline]
    fn modify(self, res: &mut Response) {
        res.body = Some(Box::new(self));
    }
}

impl Modifier<Response> for String {
    #[inline]
    fn modify(self, res: &mut Response) {
        self.into_bytes().modify(res);
    }
}

impl Modifier<Response> for Vec<u8> {
    #[inline]
    fn modify(self, res: &mut Response) {
        res.headers.set(headers::ContentLength(self.len() as u64));
        res.body = Some(Box::new(self));
    }
}

impl<'a> Modifier<Response> for &'a str {
    #[inline]
    fn modify(self, res: &mut Response) {
        self.to_owned().modify(res);
    }
}

impl<'a> Modifier<Response> for &'a [u8] {
    #[inline]
    fn modify(self, res: &mut Response) {
        self.to_vec().modify(res);
    }
}

impl Modifier<Response> for File {
    fn modify(self, res: &mut Response) {
        // Set the content type based on the file extension if a path is available.
        if let Ok(metadata) = self.metadata() {
            res.headers.set(headers::ContentLength(metadata.len()));
        }

        res.body = Some(Box::new(self));
    }
}

impl<'a> Modifier<Response> for &'a Path {
    /// Set the body to the contents of the File at this path.
    ///
    /// ## Panics
    ///
    /// Panics if there is no file at the passed-in Path.
    fn modify(self, res: &mut Response) {
        File::open(self)
            .expect(&format!("No such file: {}", self.display()))
            .modify(res);

        let mime = mime_for_path(self);
        res.set_mut(mime);
    }
}

impl Modifier<Response> for PathBuf {
    /// Set the body to the contents of the File at this path.
    ///
    /// ## Panics
    ///
    /// Panics if there is no file at the passed-in Path.
    #[inline]
    fn modify(self, res: &mut Response) {
        self.as_path().modify(res);
    }
}

impl Modifier<Response> for status::Status {
    fn modify(self, res: &mut Response) {
        res.status = Some(self);
    }
}

/// A modifier for changing headers on requests and responses.
#[derive(Clone)]
pub struct Header<H: headers::Header + headers::HeaderFormat>(pub H);

impl<H> Modifier<Response> for Header<H>
where H: headers::Header + headers::HeaderFormat {
    fn modify(self, res: &mut Response) {
        res.headers.set(self.0);
    }
}

impl<'a, 'b, H> Modifier<Request<'a, 'b>> for Header<H>
where H: headers::Header + headers::HeaderFormat {
    fn modify(self, res: &mut Request) {
        res.headers.set(self.0);
    }
}

/// A modifier for creating redirect responses.
pub struct Redirect(pub Url);

impl Modifier<Response> for Redirect {
    fn modify(self, res: &mut Response) {
        let Redirect(url) = self;
        res.headers.set(headers::Location(url.to_string()));
    }
}

/// A modifier for creating redirect responses.
pub struct RedirectRaw(pub String);

impl Modifier<Response> for RedirectRaw {
    fn modify(self, res: &mut Response) {
        let RedirectRaw(path) = self;
        res.headers.set(headers::Location(path));
    }
}

fn mime_for_path(path: &Path) -> Mime {
    guess_mime_type_opt(path)
        .unwrap_or_else(|| Mime(TopLevel::Text, SubLevel::Plain, vec![]))
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mime_for_path() {
        assert_eq!(mime_for_path(Path::new("foo.txt")),
                   "text/plain".parse().unwrap());
        assert_eq!(mime_for_path(Path::new("foo.jpg")),
                   "image/jpeg".parse().unwrap());
        assert_eq!(mime_for_path(Path::new("foo.zip")),
                   "application/zip".parse().unwrap());
        assert_eq!(mime_for_path(Path::new("foo")),
                   "text/plain".parse().unwrap());
    }
}
