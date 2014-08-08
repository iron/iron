#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/mount")]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(unused_result, unnecessary_qualification, non_camel_case_types,
        unused_variable, unnecessary_typecast)]
#![feature(macro_rules)]

//! `Mount` provides mounting middleware for the Iron framework.

extern crate http;
extern crate iron;
extern crate regex;
extern crate url;
extern crate collections;

pub use mount::{Mount, OriginalUrl};

mod mount;

