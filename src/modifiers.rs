//! Defines a series of convenience modifiers for editing Responses
//!
//! Modifiers can be used to edit Responses through the owning
//! method `set` or the mutating `set_mut`, both of which are
//! defined in the `Set` trait.
//!
//! Instead of having a combinatorial explosion of Response methods
//! and constructors, this provides a series of modifiers that
//! can be used through the `Set` trait.
//!
//! For instance, instead of `Response::redirect` constructing a
//! redirect response, we provide a `Redirect` modifier, so you
//! can just do:
//!
//! ```
//! # use iron::prelude::*;
//! # use iron::status;
//! # use iron::modifiers::Redirect;
//! # use iron::Url;
//!
//! let url = Url::parse("http://doc.rust-lang.org").unwrap();
//! Response::with((status::Found, Redirect(url)));
//! ```
//!
//! This is more extensible as it allows you to combine
//! arbitrary modifiers without having a massive number of
//! Response constructors.
//!
//! For more information about the modifier system, see
//! [rust-modifier](https://github.com/reem/rust-modifier).

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use modifier::Modifier;

use hyper::mime::Mime;

use {status, headers, Request, Response, Set, Url};

use mime_types;
use response::{WriteBody, BodyReader};

lazy_static! {
    static ref MIME_TYPES: mime_types::Types = mime_types::Types::new().unwrap();
}

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

        let mime_str = MIME_TYPES.mime_for_path(self);
        let _ = mime_str.parse().map(|mime: Mime| res.set_mut(mime));
    }
}

impl Modifier<Response> for PathBuf {
    /// Set the body to the contents of the File at this path.
    ///
    /// ## Panics
    ///
    /// Panics if there is no file at the passed-in Path.
    fn modify(self, res: &mut Response) {
        File::open(&self)
            .expect(&format!("No such file: {}", self.display()))
            .modify(res);
    }
}

impl Modifier<Response> for status::Status {
    fn modify(self, res: &mut Response) {
        res.status = Some(self);
    }
}

/// A modifier for changing headers on requests and responses.
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
pub struct RedirectRaw(String);

impl Modifier<Response> for RedirectRaw {
    fn modify(self, res: &mut Response) {
        let RedirectRaw(path) = self;
        res.headers.set(headers::Location(path));
    }
}
