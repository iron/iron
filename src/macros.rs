//! Helper macros. Note that these are relatively new and may change in a later version.
//!
//! The idea is to use `itry` for internal server operations which can't be recovered from, and
//! `iexpect` for validating user input. Note that this kind of usage is completely non-normative.
//! Feedback about actual usability and usage is appreciated.

/// Like `try!()`, but wraps the error value in `IronError`. To be used in
/// request handlers.
///
/// The second (optional) parameter is any [modifier](modifiers/index.html).
/// The default modifier is `status::InternalServerError`.
///
///
/// ```ignore
/// let f = itry!(fs::File::create("foo.txt"), request, status::BadRequest);
/// let f = itry!(fs::File::create("foo.txt"), request, (status::NotFound, "Not Found"));
/// let f = itry!(fs::File::create("foo.txt"), request);  // HTTP 500
/// ```
///
#[macro_export]
macro_rules! itry {
    ($result:expr, $req:expr) => (itry!($result, $req, $crate::status::InternalServerError));

    ($result:expr, $req:expr, $modifier:expr) => (match $result {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => return ::std::boxed::Box::new(::futures::future::err(
            $crate::IronError::new(err, $req, $modifier))) as $crate::BoxIronFuture<($crate::Request, $crate::Response)>
    })
}

/// Unwrap the given `Option` or return a `Ok(Response::new())` with the given
/// modifier. The default modifier is `status::BadRequest`.
#[macro_export]
macro_rules! iexpect {
    ($option:expr, $req:expr) => (iexpect!($option, $req, $crate::status::BadRequest));
    ($option:expr, $req:expr, $modifier:expr) => (match $option {
        ::std::option::Option::Some(x) => x,
        ::std::option::Option::None => return ::std::boxed::Box::new(::futures::future::ok(
            ($req, $crate::response::Response::with($modifier),))) as $crate::BoxIronFuture<($crate::Request, $crate::Response)>
    })
}
