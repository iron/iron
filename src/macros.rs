//! Helper macros. Note that these are relatively new and may change in a later version.
//!
//! The idea is to use `itry` for internal server operations which can't be recovered from, and
//! `iexpect` for validating user input. Note that this kind of usage is completely non-normative.
//! Feedback about actual usability and usage is appreciated.

/// Like `try!()`, but wraps the error value in `IronError`. To be used in
/// request handlers.
///
/// The second (optional) parameter is any [modifier](modifiers/index.html).
/// The default modifier is `StatusCode::INTERNAL_SERVER_ERROR`.
///
///
/// ```ignore
/// let f = itry!(fs::File::create("foo.txt"), StatusCode::BAD_REQUEST);
/// let f = itry!(fs::File::create("foo.txt"), (StatusCode::NOT_FOUND, "Not Found"));
/// let f = itry!(fs::File::create("foo.txt"));  // HTTP 500
/// ```
///
#[macro_export]
macro_rules! itry {
    ($result:expr) => {
        itry!($result, $crate::StatusCode::INTERNAL_SERVER_ERROR)
    };

    ($result:expr, $modifier:expr) => {
        match $result {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => {
                return ::std::result::Result::Err($crate::IronError::new(err, $modifier))
            }
        }
    };
}

/// Unwrap the given `Option` or return a `Ok(Response::new())` with the given
/// modifier. The default modifier is `StatusCode::BAD_REQUEST`.
#[macro_export]
macro_rules! iexpect {
    ($option:expr) => {
        iexpect!($option, $crate::StatusCode::BAD_REQUEST)
    };
    ($option:expr, $modifier:expr) => {
        match $option {
            ::std::option::Option::Some(x) => x,
            ::std::option::Option::None => {
                return ::std::result::Result::Ok($crate::response::Response::with($modifier))
            }
        }
    };
}
