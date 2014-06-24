#![crate_id = "mount"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(unused_result, unused_result, unnecessary_qualification,
        non_camel_case_types, unused_variable, unnecessary_typecast)]
#![feature(macro_rules)]

//! `Mount` provides mounting middleware for the Iron framework.

extern crate http;
extern crate iron;
extern crate regex;

pub use Mount = mount::Mount;

mod mount;

