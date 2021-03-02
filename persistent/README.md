persistent [![Build Status](https://secure.travis-ci.org/iron/persistent.png?branch=master)](https://travis-ci.org/iron/persistent)
====

> Persistent storage as middleware for the [Iron](https://github.com/iron/iron) web framework.

- Share persistent data across requests
- Read or modify locally stored data

Use this if you are currently thinking about using `std::sync::Arc` to share
state between request handlers.

## Installation

If you're using a `Cargo.toml` to manage dependencies, just add persistent to the toml:

```toml
[dependencies]
persistent = "x.y.z"  # Insert current version here
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](https://docs.rs/persistent)

Along with the [online documentation](https://docs.rs/persistent),
you can build a local copy with `make doc`.

## [Examples](/examples)

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.
