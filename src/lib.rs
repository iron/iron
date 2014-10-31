#![crate_name = "router"]
#![license = "MIT"]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(phase, globs)]

//! `Router` provides a fast router handler for the Iron web framework.

extern crate http;
extern crate iron;
extern crate "route-recognizer" as recognizer;
extern crate typemap;

#[cfg(test)] extern crate test;

pub use router::Router;
pub use recognizer::Params;

mod router;
