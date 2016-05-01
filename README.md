logger
====

> [Morgan](https://github.com/expressjs/morgan)-inspired logging middleware for the [Iron](https://github.com/iron/iron) web framework.

This is a forked edition of original [logger](https://github.com/iron/logger)

## Example

```rust
extern crate iron;
extern crate logger;

use iron::prelude::*;
use logger::Logger;

fn main() {
    let (logger_before, logger_after) = Logger::new(None);

    let mut chain = Chain::new(no_op_handler);

    // Link logger_before as your first before middleware.
    chain.link_before(logger_before);

    // Link logger_after as your *last* after middleware.
    chain.link_after(logger_after);

    Iron::new(chain).http("127.0.0.1:3000").unwrap();
}

fn no_op_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}
```

## Overview

Logger prints request and response information to the configured log, using either a default format or a custom format string.

Format strings can specify fields to be logged (ANSI terminal colors and attributes is no longer supported).

## Installation

If you're using a `Cargo.toml` to manage dependencies, just add logger to the toml:

```toml
[dependencies.logger]

git = "https://github.com/alexander-irbis/logger.git"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://docs.ironframework.io/logger) (unmaintained).

You can build a local copy with `make doc`.

## [Examples](/examples)

