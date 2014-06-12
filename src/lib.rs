#![crate_id = "iron"]
#![comment = "Rapid Web Development in Rust"]
#![license = "MIT"]

//#![deny(missing_doc)]
#![deny(unused_result)]
#![deny(unnecessary_qualification)]
#![deny(non_camel_case_types)]
#![deny(unused_variable)]
#![deny(deprecated_owned_vector)]
#![deny(unnecessary_typecast)]

#![feature(macro_rules, phase)]
//! The main crate for the Iron library.

extern crate http;

pub mod request;
pub mod response;
pub mod furnace;
pub mod ingot;
pub mod iron;
