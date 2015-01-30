#![deny(missing_docs, warnings)]
#![feature(core)]
#![feature(std_misc)]

//! `Router` provides a fast router handler for the Iron web framework.

extern crate iron;
extern crate "route-recognizer" as recognizer;

pub use router::Router;
pub use recognizer::Params;

mod router;


