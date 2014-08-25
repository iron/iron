#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/router")]
#![crate_name = "router"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]
#![feature(phase, globs)]

//! `Router` provides a fast router handler for the Iron web framework.

extern crate http;
extern crate iron;
extern crate recognizer = "route-recognizer";
extern crate typemap;

#[cfg(test)] extern crate test;

pub use router::Router;
pub use recognizer::Params;

mod router;
