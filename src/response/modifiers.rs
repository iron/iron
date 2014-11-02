//! Defines a series of convenience modifiers for editing Responses

use std::str::StrAllocating;
use std::io::{File, MemReader};
use std::path::Path;

use modifier::Modifier;
use http::headers::content_type::MediaType;
use content_type::get_content_type;
use {status, Response};

/// A response modifier for setting the content-type header.
pub struct ContentType(pub MediaType);

impl ContentType {
    /// Create a new ContentType modifier from the parts of a content-type header value.
    #[inline]
    pub fn new<S: StrAllocating, S1: StrAllocating>(type_: S, subtype: S1) -> ContentType {
        ContentType(MediaType::new(type_.into_string(), subtype.into_string(), vec![]))
    }
}

impl Modifier<Response> for ContentType {
    #[inline]
    fn modify(self, mut res: Response) -> Response {
        let ContentType(media) = self;
        res.headers.content_type = Some(media);
        res
    }
}

/// A response modifier for seeting the body of a response.
pub struct Body<B: Bodyable>(B);

impl<B: Bodyable> Modifier<Response> for Body<B> {
    #[inline]
    fn modify(self, mut res: Response) -> Response {
        let Body(b) = self;
        b.set_body(&mut res);
        res
    }
}

/// A modifier that can be used to set the body of a response.
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
        res.headers.content_length = Some(self.len());
        res.body = Some(box MemReader::new(self) as Box<Reader + Send>);
    }
}

impl<'a> Bodyable for &'a str {
    #[inline]
    fn set_body(self, res: &mut Response) {
        self.into_string().set_body(res);
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
        res.headers.content_type = self.path().extension_str().and_then(get_content_type);
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
pub struct Status(pub status::Status);

impl Modifier<Response> for Status {
    fn modify(self, mut res: Response) -> Response {
        let Status(status) = self;
        res.status = Some(status);
        res
    }
}

