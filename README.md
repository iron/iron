Router [![Build Status](https://secure.travis-ci.org/iron/router.png?branch=master)](https://travis-ci.org/iron/router)
====

> Routing handler for the [Iron](https://github.com/iron/iron) web framework.

Router is a fast, convenient, and flexible routing middleware for Iron. It
allows complex glob patterns and named url parameters and also allows handlers
to be any Handler, including all Chains.

## Example

```rust
fn main() {
    let mut router = Router::new();
    router.get("/", handler);
    router.get("/:query", handler);

    Iron::new(router).listen(Ipv4Addr(127, 0, 0, 1), 3000);

    fn handler(req: &mut Request) -> IronResult<Response> {
        let ref query = req.extensions.find::<Router>().unwrap().find("query").unwrap_or("/");
        Ok(Response::with(status::Ok, *query))
    }
}
```

## Overview

Router is a part of Iron's [core bundle](https://github.com/iron/core).

- Route client requests based on their paths
- Parse parameters and provide them to other middleware/handlers

## Installation

If you're using cargo, just add router to your `Cargo.toml`.

```toml
[dependencies.router]

git = "https://github.com/iron/router.git"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://ironframework.io/doc/router)

Along with the [online documentation](http://ironframework.io/doc/router),
you can build a local copy with `make doc`.

## [Examples](/examples)

## Get Help

One of us is usually on `#iron` on the mozilla irc.
Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.

