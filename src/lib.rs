#![crate_name = "router"]
#![license = "MIT"]
#![deny(missing_docs, warnings)]
#![feature(phase, globs)]

//! `Router` provides a fast router handler for the Iron web framework.

extern crate iron;
extern crate "route-recognizer" as recognizer;

#[cfg(test)] extern crate test;

pub use router::Router;
pub use recognizer::Params;

mod router;

