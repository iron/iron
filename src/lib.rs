#![crate_name = "router"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(unused_result, unused_result, unnecessary_qualification,
        non_camel_case_types, unused_variable, unnecessary_typecast)]
#![feature(phase, globs)]

//! `Router` provides a fast router middleware for the Iron web framework.

extern crate http;
extern crate iron;
extern crate recognizer = "route-recognizer";

#[cfg(test)] extern crate test;

pub use router::Router;
pub use recognizer::Params;

mod router;

