#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/iron")]
#![crate_id = "iron"]
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

extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate http;
extern crate anymap;
#[cfg(test)]
extern crate test;

pub use request::Request;
pub use response::Response;

pub use iron::{Iron, ServerT};
pub use middleware::{Middleware, Status, Continue, Unwind, FromFn};

pub use chain::Chain;
pub use chain::stackchain::StackChain;

pub use alloy::Alloy;

pub mod mixin;
mod request;
mod response;
mod middleware;
mod alloy;
mod chain;
mod iron;
