#![crate_id = "iron"]
#![comment = "Rapid Web Development in Rust"]
#![license = "MIT"]

#![deny(missing_doc)]
#![deny(unused_result)]
#![deny(unnecessary_qualification)]
#![deny(non_camel_case_types)]
#![deny(unused_variable)]
#![deny(unnecessary_typecast)]

#![feature(macro_rules, phase)]
//! The main crate for the Iron library.

extern crate http;
extern crate anymap;

pub use request::Request;
pub use response::Response;

pub use iron::{Iron, ServerT};
pub use middleware::Middleware;

pub use furnace::Furnace;
pub use furnace::stackfurnace::StackFurnace;

pub use alloy::Alloy;

pub mod request;
pub mod response;
pub mod middleware;
pub mod alloy;
pub mod furnace;
pub mod iron;

