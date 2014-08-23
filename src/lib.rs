#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/iron")]
#![crate_name = "iron"]
#![comment = "Rapid Web Development in Rust"]
#![license = "MIT"]

#![deny(missing_doc)]
#![deny(warnings)]

#![feature(macro_rules, phase, globs)]
//! The main crate for the Iron library.
//!
//! Iron is a high level web framework built in and for Rust.
//!
//! Iron does not come bundled with any middleware - instead, Iron
//! provides a robust and efficient framework for creating and
//! plugging in middleware.
//!
//! Obligatory example:
//!
//! ```ignore
//! #[deriving(Clone)]
//! struct ResponseTime {
//!     entry_time: u64
//! }
//!
//! impl ResponseTime { fn new() -> ResponseTime { ResponseTime { entry_time: 0u64 } } }
//!
//! impl Middleware for ResponseTime {
//!     fn enter(&mut self, _req: &mut Request, _res: &mut Response) -> Status {
//!         self.entry_time = precise_time_ns();
//!         Continue
//!     }
//!
//!     fn exit(&mut self, _req: &mut Request, _res: &mut Response) -> Status {
//!         let delta = precise_time_ns() - self.entry_time;
//!         println!("Request took: {} ms", (delta as f64) / 1000000.0);
//!         Continue
//!     }
//! }
//!
//! // ...
//! server.chain.link(ResponseTime::new());
//! // ...
//! ```

// Stdlib dependencies
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate log;
#[cfg(test)] extern crate test;

// Third party packages
extern crate contenttype;
extern crate http;
extern crate typemap;
extern crate plugin;
extern crate error;
extern crate rust_url = "url";

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

// Expose `GetCached` as `Plugin` so users can do `use iron::Plugin`.
pub use plugin::GetCached as Plugin;

// Return type of many methods
pub type IronResult<T> = Result<T, Box<error::Error>>;

// Internal modules
mod request;
mod response;
mod iron;
mod middleware;

