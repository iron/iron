#![crate_name = "persistent"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(unused_result, unused_result, unnecessary_qualification,
        non_camel_case_types, unused_variable, unnecessary_typecast)]

//! A set of middleware for sharing data between requests in the Iron
//! framework.

extern crate iron;

pub use persistent::Persistent;
pub use shared::Shared;
pub use mixin::SharedLink;

mod persistent;
mod shared;
mod mixin;

