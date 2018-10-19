/// Create and populate a router.
///
/// ```ignore
/// let router = router!(index: get  "/"       => index,
///                      query: get  "/:query" => queryHandler,
///                      post:  post "/"       => postHandler);
/// ```
///
/// Is equivalent to:
///
/// ```ignore
/// let mut router = Router::new();
/// router.get("/", index, "index");
/// router.get("/:query", queryHandler, "query");
/// router.post("/", postHandler, "post");
/// ```
///
/// The method name must be lowercase, supported methods:
///
/// `get`, `post`, `put`, `delete`, `head`, `patch`, `options` and `any`.
#[macro_export]
macro_rules! router {
    ($($route_id:ident: $method:ident $glob:expr => $handler:expr),+ $(,)*) => ({
        let mut router = $crate::Router::new();
        $(router.$method($glob, $handler, stringify!($route_id));)*
        router
    });
}

/// Generate a URL based off of the requested one.
///
/// ```ignore
/// url_for!(request, "foo",
///          "query" => "test",
///          "extraparam" => "foo")
/// ```
///
/// Is equivalent to:
///
/// ```ignore
/// router::url_for(request, "foo", {
///     let mut rv = ::std::collections::HashMap::new();
///     rv.insert("query".to_owned(), "test".to_owned());
///     rv.insert("extraparam".to_owned(), "foo".to_owned());
///     rv
/// })
/// ```
#[macro_export]
macro_rules! url_for {
    ($request:expr, $route_id:expr $(,$key:expr => $value:expr)* $(,)*) => (
        $crate::url_for($request, $route_id, {
            // Underscore-prefix suppresses `unused_mut` warning
            // Also works on stable rust!
            let mut _params = ::std::collections::HashMap::<String, String>::new();
            $(_params.insert($key.into(), $value.into());)*
            _params
        })
    )
}

#[cfg(test)]
mod tests {
    use iron::{Response, Request, IronResult};

    //simple test to check that all methods expand without error
    #[test]
    fn methods() {
        fn handler(_: &mut Request) -> IronResult<Response> {Ok(Response::new())}
        let _ = router!(a: get     "/foo" => handler,
                        b: post    "/bar/" => handler,
                        c: put     "/bar/baz" => handler,
                        d: delete  "/bar/baz" => handler,
                        e: head    "/foo" => handler,
                        f: patch   "/bar/baz" => handler,
                        g: options "/foo" => handler,
                        h: any     "/" => handler);
    }
}
