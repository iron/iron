#![crate_name = "mount"]
#![license = "MIT"]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(macro_rules)]

//! `Mount` provides mounting middleware for the Iron framework.

extern crate http;
extern crate iron;
extern crate regex;
extern crate url;
extern crate typemap;
extern crate trie;

pub use mount::{Mount, OriginalUrl};

mod mount;
