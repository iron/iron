#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/router")]
#![crate_id = "router"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(unused_result, unused_result, unnecessary_qualification,
        non_camel_case_types, unused_variable, unnecessary_typecast)]
#![feature(phase, globs)]

//! `Router` provides a fast router middleware for the Iron web framework.

extern crate http;
extern crate iron;
extern crate regex;
#[phase(plugin, link)] extern crate log;
#[phase(plugin)] extern crate regex_macros;

#[cfg(test)] extern crate test;

pub use router::Router;
pub use router::params::Params;

mod router;

