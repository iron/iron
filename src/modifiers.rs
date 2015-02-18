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

use std::old_io::{File, MemReader};
use std::old_path::Path;

use modifier::Modifier;

use hyper::mime::Mime;

use {status, headers, Response, Url};

impl Modifier<Response> for Mime {
    #[inline]
    fn modify(self, res: &mut Response) {
        res.headers.set(headers::ContentType(self))
    }
}

impl Modifier<Response> for Box<Reader + Send> {
    #[inline]
    fn modify(self, res: &mut Response) {
        res.body = Some(self);
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
        (self.len() as u64).modify(res);
        res.body = Some(Box::new(MemReader::new(self)) as Box<Reader + Send>);
    }
}

impl<'a> Modifier<Response> for &'a str {
    #[inline]
    fn modify(self, res: &mut Response) {
        self.to_string().modify(res);
    }
}

impl<'a> Modifier<Response> for &'a [u8] {
    #[inline]
    fn modify(self, res: &mut Response) {
        self.to_vec().modify(res);
    }
}

impl Modifier<Response> for u64 {
    #[inline]
    fn modify(self, res: &mut Response) {
        res.headers.set(headers::ContentLength(self));
    }
}

impl Modifier<Response> for File {
    #[inline]
    fn modify(self, res: &mut Response) {
        // Also set the content type.
        // self.path().extension_str()
        //     .and_then(get_content_type)
        //     .and_then(|ct| { res.set_mut(ct) });

        if let Ok(stat) = self.stat() {
            stat.size.modify(res);
        }

        res.body = Some(Box::new(self) as Box<Reader + Send>);
    }
}

impl Modifier<Response> for Path {
    /// Set the body to the contents of the File at this path.
    ///
    /// ## Panics
    ///
    /// Panics if there is no file at the passed-in Path.
    fn modify(self, res: &mut Response) {
        File::open(&self)
            .ok().expect(format!("No such file: {}", self.display()).as_slice())
            .modify(res);
    }
}

impl Modifier<Response> for status::Status {
    fn modify(self, res: &mut Response) {
        res.status = Some(self);
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

