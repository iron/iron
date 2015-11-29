/// Create and populate a router.
///
/// ```ignore
/// let router = router!(get  "/"       => index,
///                      get  "/:query" => queryHandler,
///                      post "/"       => postHandler);
/// ```
///
/// Is equivalent to:
///
/// ```ignore
///    let mut router = Router::new();
///    router.get("/", index);
///    router.get("/:query", queryHandler);
///    router.post("/", postHandler);
/// ```
///
/// The method name must be lowercase, supported methods:
///
/// `get`, `post`, `put`, `delete`, `head`, `patch`, `options` and `any`.
#[macro_export]
macro_rules! router {
    ($($method:ident $glob:expr => $handler:expr),+ $(,)*) => ({
        let mut router = $crate::Router::new();
        $(router.$method($glob, $handler);)*
        router
    });
}

#[cfg(test)]
mod tests {
    use iron::{Response, Request, IronResult};

    //simple test to check that all methods expand without error
    #[test]
    fn methods() {
        fn handler(_: &mut Request) -> IronResult<Response> {Ok(Response::new())}
        let _ = router!(get     "/foo" => handler,
                        post    "/bar/" => handler,
                        put     "/bar/baz" => handler,
                        delete  "/bar/baz" => handler,
                        head    "/foo" => handler,
                        patch   "/bar/baz" => handler,
                        options "/foo" => handler,
                        any     "/" => handler);
    }
}
