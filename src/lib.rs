#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/persistent")]
#![crate_id = "persistent"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(unused_result, unused_result, unnecessary_qualification,
        non_camel_case_types, unused_variable, unnecessary_typecast)]

//! A set of middleware for sharing data between requests in the Iron
//! framework.

extern crate iron;

pub use persistent::Persistent;
pub use shared::Shared;

/// Exposes the `Persistent` middleware, for sharing a single piece of data
/// between requests.
pub mod persistent;

/// Exposes the `Shared` middleware, which wraps `ShareableMiddleware` so that
/// they are not cloned for each request and improves performance by removing
/// unnecessary cloning of immutable data.
pub mod shared;

/// Exposes a `link_shared` method on `Chain` objects, providing a convenient
/// way to add shared middleware using `Shared`.
pub mod mixin;

