logger [![Build Status](https://secure.travis-ci.org/iron/logger.png?branch=master)](https://travis-ci.org/iron/logger)
====

> [Morgan](https://github.com/expressjs/morgan)-inspired logging middleware for the [Iron](https://github.com/iron/iron) web framework.

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

Logger is a part of Iron's [core bundle](https://github.com/iron/core).

- Logger prints request and response information to the terminal, using either a default format or a custom format string.
- Format strings can specify fields to be logged as well as ANSI terminal colors and attributes.

## Installation

If you're using a `Cargo.toml` to manage dependencies, just add logger to the toml:

```toml
[dependencies.logger]

git = "https://github.com/iron/logger.git"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://docs.ironframework.io/logger)

Along with the [online documentation](http://docs.ironframework.io/logger),
you can build a local copy with `make doc`.

## [Examples](/examples)

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.

