#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/iron")]
#![deny(missing_docs)]
#![deny(warnings)]
#![allow(unstable)]

#![feature(unboxed_closures, slicing_syntax)]

//! The main crate for the Iron library.
//!
//! Iron is a high level web framework built in and for Rust.
//!
//! Iron provides a robust and efficient framework
//! for creating and plugging in middleware.
//!
//! Obligatory Hello World:
//!
//! ```ignore
//! fn hello_world(req: &mut Request) -> IronResult<Response> {
//!   Response::new().set(Status(status::Ok)).set(Body("Hello World!"))
//! }
//!
//! Iron::new(hello_world).listen(Ipv4Addr(127, 0, 0, 1), 3000);
//! ```

// Stdlib dependencies
#[macro_use] extern crate log;
#[cfg(test)] extern crate test;

// Third party packages
extern crate hyper;
extern crate "typemap" as tmap;
extern crate plugin;
extern crate "modifier" as modfier;
extern crate error;
extern crate url;

// Request + Response
pub use request::{Request, Url};
pub use response::Response;

// Middleware system
pub use middleware::{BeforeMiddleware, AfterMiddleware, AroundMiddleware,
                     Handler, Chain, ChainBuilder};

// Server
pub use iron::Iron;

// Extensions
pub use typemap::TypeMap;

// Headers
pub use hyper::header::common as headers;
pub use hyper::header::Headers;

// Expose `Pluggable` as `Plugin` so users can do `use iron::Plugin`.
pub use plugin::Pluggable as Plugin;

// Expose modifiers.
pub use modifier::Set;

// Errors
pub use error::Error;

// Mime types
pub use hyper::mime;

// Return type of many methods

/// The type of Errors inside and when using Iron.
pub type IronError = Box<Error>;

/// The Result alias used throughout Iron and in clients of Iron.
pub type IronResult<T> = Result<T, IronError>;

/// A module meant to be glob imported when using Iron, for instance:
///
/// ```{ignore}
/// #![feature(globs)]
/// use iron::prelude::*;
/// ```
///
/// This module contains several important traits that provide many
/// of the convenience methods in Iron, as well as `Request`, `Response`
/// `IronResult`, `IronError` and `Iron`.
pub mod prelude {
    pub use {Set, Plugin, Chain, Request, Response,
             IronResult, IronError, Iron};
}

/// Re-exports from the TypeMap crate.
pub mod typemap {
    pub use tmap::{TypeMap, Key};
}

/// Re-exports from the Modifier crate.
pub mod modifier;

/// Status Codes
pub mod status {
    pub use hyper::status::StatusCode as Status;
    pub use hyper::status::StatusCode::*;
}

/// HTTP Methods
pub mod method {
    pub use hyper::method::Method;
    pub use hyper::method::Method::*;
}

// Publicized to show the documentation
pub mod middleware;

// Response utilities
pub mod response;

// Request and Response Modifiers
pub mod modifiers;

// Internal modules
mod request;
mod iron;

