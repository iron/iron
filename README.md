router [![Build Status](https://secure.travis-ci.org/iron/router.png?branch=master)](https://travis-ci.org/iron/router)
====

> Routing middleware for the [Iron](https://github.com/iron/iron) web framework.

Router is a fast, convenient, and flexible routing middleware for Iron. It
allows complex glob patterns and named url parameters and also allows handlers
to be any Middleware - including Chain, which provides enormous amounts of
flexibility for handling and dispatching requests.

## Example

```rust
fn main() {
    let mut router = Router::new();
    router.route( // Setup our route.
        Get,
        "/:class/:id".to_string(),
        vec!["class".to_string(), "id".to_string()],
        echo_to_term);

    let mut server: Server = Iron::new();
    server.chain.link(router); // Add middleware to the server's stack
    server.listen(::std::io::net::ip::Ipv4Addr(127, 0, 0, 1), 3000);
}

fn echo_to_term(_: &mut Request, res: &mut Response, alloy: &mut Alloy) {
    let query = alloy.find::<Params>().unwrap();
    println!("Class: {}\t id: {}",
             query.get("class").unwrap(), query.get("id").unwrap());
}
```

## Overview

router is a part of Iron's [core bundle](https://github.com/iron/core).

- Route client requests based on their paths
- Parse parameters and provide them to other middleware/handlers

## Installation

If you're using a `Cargo.toml` to manage dependencies, just add router to the toml:

```toml
[dependencies.router]

git = "https://github.com/iron/router.git"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://docs.ironframework.io/core/router)

Along with the [online documentation](http://docs.ironframework.io/core/router),
you can build a local copy with `make doc`.

## [Examples](/examples)

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.
