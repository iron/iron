#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/persistent")]
#![crate_id = "persistent"]
#![license = "MIT"]
//#![deny(missing_doc)]
#![deny(unused_result, unused_result, unnecessary_qualification,
        non_camel_case_types, unused_variable, unnecessary_typecast)]

extern crate iron;

pub use persistent::Persistent;
pub use shared::Shared;

pub mod persistent;
pub mod shared;
pub mod mixin;

