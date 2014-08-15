#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/iron")]
#![crate_name = "iron"]
#![comment = "Rapid Web Development in Rust"]
#![license = "MIT"]

#![deny(missing_doc)]
#![deny(unused_result)]
#![deny(unnecessary_qualification)]
#![deny(non_camel_case_types)]
#![deny(unused_variable)]
#![deny(unnecessary_typecast)]

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
//!         println!("Request took: {} ms", (delta as f64) / 100000.0);
//!         Continue
//!     }
//! }
//!
//! // ...
//! server.chain.link(ResponseTime::new());
//! // ...
//! ```

extern crate regex;
#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate log;

extern crate plugin;
extern crate contenttype;
extern crate http;
extern crate anymap;
// Rename the URL crate to avoid clashes with the `url` module.
extern crate rust_url = "url";
#[cfg(test)]
extern crate test;

pub use request::Request;
pub use response::Response;

pub use iron::{Iron, Server};
pub use middleware::{Middleware, Status, Continue, Unwind, Error, FromFn};

pub use chain::Chain;
pub use chain::stackchain::StackChain;

pub use anymap::AnyMap;

#[deprecated = "Alloy is deprecated - use AnyMap instead."]
pub use Alloy = anymap::AnyMap;

pub use url::Url;

pub use plugin::PluginFor;

mod request;
mod response;
mod middleware;
mod chain;
mod iron;
mod url;
