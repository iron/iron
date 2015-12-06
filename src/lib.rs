#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! `Router` provides fast and flexible routing for Iron.

extern crate iron;
extern crate route_recognizer as recognizer;

pub use router::{Router, NoRoute, TrailingSlash};
pub use recognizer::Params;

mod router;
mod macros;
