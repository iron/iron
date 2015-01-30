#![deny(missing_docs)]
#![feature(core, std_misc)]
#![cfg_attr(test, deny(warnings))]

//! `Router` provides fast and flexible routing for Iron.

extern crate iron;
extern crate "route-recognizer" as recognizer;

pub use router::Router;
pub use recognizer::Params;

mod router;


