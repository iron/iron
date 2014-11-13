#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/iron")]
#![crate_name = "iron"]
#![comment = "Rapid Web Development in Rust"]
#![license = "MIT"]

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(macro_rules, phase, globs, unboxed_closures, slicing_syntax)]

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
#[phase(plugin, link)] extern crate log;
#[cfg(test)] extern crate test;
extern crate serialize;

// Third party packages
extern crate content_type;
extern crate http;
extern crate "typemap" as tmap;
extern crate plugin;
extern crate modifier;
extern crate error;
extern crate taskpool;
extern crate "url" as rust_url;

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

// Status codes and Methods.
pub use http::status;
pub use http::method;

// Headers
pub use http::headers;

// Expose `GetCached` as `Plugin` so users can do `use iron::Plugin`.
pub use plugin::GetCached as Plugin;

// Expose modifiers.
pub use modifier::Set;

// Errors
pub use error::{Error, ErrorRefExt};

// Return type of many methods
pub type IronError = Box<Error>;
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
    pub use {Set, Plugin, ErrorRefExt, Chain, Request,
             Response, IronResult, IronError, Iron};
}

/// Re-exports from the TypeMap crate.
pub mod typemap {
    pub use tmap::{TypeMap, Assoc};
}

// Publicized to show the documentation
pub mod middleware;

// Common Errors
pub mod errors;

// Response utilities
pub mod response;

// Internal modules
mod request;
mod iron;

