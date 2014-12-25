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
//! ```rust,ignore
//! Response::new()
//!     .set(Status(status))
//!     .set(Redirect(url));
//! ```
//!
//! This is more extensible as it allows you to combine
//! arbitrary modifiers without having a massive number of
//! Response constructors.
//!
//! For more information about the modifier system, see
//! [rust-modifier](https://github.com/reem/rust-modifier).

use std::io::{File, MemReader};
use std::path::Path;

use modifier::Modifier;
// use content_type::get_content_type;

use hyper::mime::Mime;

use {status, headers, Response, Url};

/// A response modifier for setting the content-type header.
pub struct ContentType(pub Mime);

impl ContentType {
    /// Create a new ContentType modifier from  a content-type header value.
    #[inline]
    pub fn new(m: Mime) -> ContentType {
        ContentType(m)
    }
}

impl Modifier<Response> for ContentType {
    #[inline]
    fn modify(self, res: &mut Response) {
        let ContentType(media) = self;
        res.headers.set(headers::ContentType(media))
    }
}

/// A response modifier for setting the body of a response.
pub struct Body<B: Bodyable>(pub B);

impl<B: Bodyable> Modifier<Response> for Body<B> {
    #[inline]
    fn modify(self, res: &mut Response) {
        let Body(b) = self;
        b.set_body(res);
    }
}

/// Something that can be used to set the body of a response.
pub trait Bodyable {
    /// Set the body of this response, possibly also setting headers.
    fn set_body(self, res: &mut Response);
}

impl Bodyable for Box<Reader + Send> {
    #[inline]
    fn set_body(self, res: &mut Response) {
        res.body = Some(self);
    }
}

impl Bodyable for String {
    #[inline]
    fn set_body(self, res: &mut Response) {
        self.into_bytes().set_body(res);
    }
}

impl Bodyable for Vec<u8> {
    #[inline]
    fn set_body(self, res: &mut Response) {
        res.headers.set(headers::ContentLength(self.len()));
        res.body = Some(box MemReader::new(self) as Box<Reader + Send>);
    }
}

impl<'a> Bodyable for &'a str {
    #[inline]
    fn set_body(self, res: &mut Response) {
        self.to_string().set_body(res);
    }
}

impl<'a> Bodyable for &'a [u8] {
    #[inline]
    fn set_body(self, res: &mut Response) {
        self.to_vec().set_body(res);
    }
}

impl Bodyable for File {
    #[inline]
    fn set_body(self, res: &mut Response) {
        // Also set the content type.
        // self.path().extension_str()
        //     .and_then(get_content_type)
        //     .and_then(|ct| {
        //         res.headers.set(headers::ContentType(ct))
        //     });
        res.body = Some(box self as Box<Reader + Send>);
    }
}

impl Bodyable for Path {
    /// Set the body to the contents of the File at this path.
    ///
    /// ## Panics
    ///
    /// Panics if there is no file at the passed-in Path.
    fn set_body(self, res: &mut Response) {
        File::open(&self)
            .ok().expect(format!("No such file: {}", self.display()).as_slice())
            .set_body(res);
    }
}

/// A modifier for setting the status of a response.
#[deriving(Copy)]
pub struct Status(pub status::Status);

impl Modifier<Response> for Status {
    fn modify(self, res: &mut Response) {
        let Status(status) = self;
        res.status = Some(status);
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

