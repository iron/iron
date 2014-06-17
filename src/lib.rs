#![crate_id = "iron-router"]
#![license = "MIT"]
//#![deny(missing_doc)]
#![deny(unused_result, unused_result, unnecessary_qualification,
        non_camel_case_types, unused_variable, unnecessary_typecast)]
#![feature(phase, globs, macro_rules)]

extern crate http;
extern crate iron;
extern crate regex;
#[phase(plugin, link)] extern crate log;
#[phase(plugin)] extern crate regex_macros;

#[cfg(test)] extern crate test;

pub use router::Router;
pub use router::Handler;
pub use router::params::Params;

mod router;

