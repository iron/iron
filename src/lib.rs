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
//! ```rust
//! #[deriving(Clone)]
//! pub struct ResponseTime { entry: u64 }
//!
//! impl ResponseTime { fn new() -> ResponseTime { ResponseTime { entry: 0u64 } } }
//!
//! // This Trait defines middleware.
//! impl MiddleWare for ResponseTime {
//!     fn enter(&mut self, _: &mut Request, _: &mut Response, _: &mut Alloy) -> Status {
//!         self.entry = precise_time_ns();
//!         Continue // Continue to other middleware in the stack
//!     }
//!
//!     fn exit(&mut self, _: &mut Request, _: &mut Response, _: &mut Alloy) -> Status {
//!         let delta = precise_time_ns() - self.entry;
//!         println!("Request took {} ms.", (delta as f64) / 100000.0)
//!         Continue
//!     }
//! }
//!
//! // ...
//! server.chain.link(ResponseTime::new());
//! // ...
//! ```
//!


extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate contenttype;
extern crate http;
extern crate anymap;
#[cfg(test)]
extern crate test;

pub use request::Request;
pub use response::Response;

pub use iron::{Iron, Server};
pub use middleware::{Middleware, Status, Continue, Unwind, FromFn};

pub use chain::Chain;
pub use chain::stackchain::StackChain;

pub use alloy::Alloy;

mod request;
mod response;
mod middleware;
mod alloy;
mod chain;
mod iron;
