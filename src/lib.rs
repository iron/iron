#![crate_name = "router"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]
#![feature(phase, globs)]

//! `Router` provides a fast router handler for the Iron web framework.

extern crate http;
extern crate iron;
extern crate recognizer = "route-recognizer";
extern crate typemap;

#[cfg(test)] extern crate test;

pub use router::Router;
pub use recognizer::Params;

mod router;
