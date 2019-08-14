#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/iron")]
#![cfg_attr(test, deny(warnings))]
#![allow(bare_trait_objects)]
#![deny(missing_docs)]

//! The main crate for Iron.
//!
//! ## Overview
//!
//! Iron is a high level web framework built in and for Rust, built on
//! [hyper](https://github.com/hyperium/hyper). Iron is designed to take advantage
//! of Rust's greatest features - its excellent type system and principled
//! approach to ownership in both single threaded and multi threaded contexts.
//!
//! Iron is highly concurrent and can scale horizontally on more machines behind a
//! load balancer or by running more threads on a more powerful machine. Iron
//! avoids the bottlenecks encountered in highly concurrent code by avoiding shared
//! writes and locking in the core framework.
//!
//! ## Hello World
//!
//! ```no_run
//! extern crate iron;
//!
//! use iron::prelude::*;
//! use iron::status;
//!
//! fn main() {
//!     Iron::new(|_: &mut Request| {
//!         Ok(Response::with((status::Ok, "Hello World!")))
//!     }).http("localhost:3000").unwrap();
//! }
//! ```
//!
//! ## Design Philosophy
//!
//! Iron is meant to be as extensible and pluggable as possible; Iron's core is
//! concentrated and avoids unnecessary features by leaving them to middleware,
//! plugins, and modifiers.
//!
//! Middleware, Plugins, and Modifiers are the main ways to extend Iron with new
//! functionality. Most extensions that would be provided by middleware in other
//! web frameworks are instead addressed by the much simpler Modifier and Plugin
//! systems.
//!
//! Modifiers allow external code to manipulate Requests and Response in an ergonomic
//! fashion, allowing third-party extensions to get the same treatment as modifiers
//! defined in Iron itself. Plugins allow for lazily-evaluated, automatically cached
//! extensions to Requests and Responses, perfect for parsing, accessing, and
//! otherwise lazily manipulating an http connection.
//!
//! Middleware are only used when it is necessary to modify the control flow of a
//! Request flow, hijack the entire handling of a Request, check an incoming
//! Request, or to do final post-processing. This covers areas such as routing,
//! mounting, static asset serving, final template rendering, authentication, and
//! logging.
//!
//! Iron comes with only basic modifiers for setting the status, body, and various
//! headers, and the infrastructure for creating modifiers, plugins, and
//! middleware. No plugins or middleware are bundled with Iron.
//!

// Stdlib dependencies
#[macro_use] extern crate log;

// Third party packages
extern crate hyper;
extern crate typemap as tmap;
extern crate plugin;
extern crate url as url_ext;
extern crate num_cpus;
extern crate mime_guess;

// Request + Response
pub use request::{Request, Url};
pub use response::Response;

// Middleware system
pub use middleware::{BeforeMiddleware, AfterMiddleware, AroundMiddleware,
                     Handler, Chain};

// Server
pub use iron::*;

// Extensions
pub use typemap::TypeMap;

// Headers
pub use hyper::header as headers;
pub use hyper::header::Headers;

// Expose `Pluggable` as `Plugin` so users can do `use iron::Plugin`.
pub use plugin::Pluggable as Plugin;

// Expose modifiers.
pub use modifier::Set;

// Errors
pub use error::Error;
pub use error::IronError;

// Mime types
pub use hyper::mime;

/// Iron's error type and associated utilities.
pub mod error;

/// The Result alias used throughout Iron and in clients of Iron.
pub type IronResult<T> = Result<T, IronError>;

/// A module meant to be glob imported when using Iron.
///
/// For instance:
///
/// ```
/// use iron::prelude::*;
/// ```
///
/// This module contains several important traits that provide many
/// of the convenience methods in Iron, as well as `Request`, `Response`
/// `IronResult`, `IronError` and `Iron`.
pub mod prelude {
    #[doc(no_inline)]
    pub use {Set, Plugin, Chain, Request, Response,
             IronResult, IronError, Iron};
}

/// Re-exports from the `TypeMap` crate.
pub mod typemap {
    pub use tmap::{TypeMap, Key};
}

/// Re-exports from the Modifier crate.
pub mod modifier {
    extern crate modifier as modfier;
    pub use self::modfier::*;
}

/// Re-exports from the url crate.
pub mod url {
    pub use url_ext::*;
}

/// Status Codes
pub mod status {
    pub use hyper::status::StatusCode as Status;
    pub use hyper::status::StatusCode::*;
    pub use hyper::status::StatusClass;
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

// Request utilities
pub mod request;

// Request and Response Modifiers
pub mod modifiers;

// Helper macros for error handling
mod macros;

mod iron;
