#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! `Router` provides fast and flexible routing for Iron.

extern crate iron;
extern crate route_recognizer as recognizer;
extern crate url;

pub use recognizer::Params;
pub use router::{NoRoute, Router, TrailingSlash};
pub use url_for::url_for;

mod macros;
mod router;
mod url_for;
