mount [![Build Status](https://secure.travis-ci.org/iron/mount.png?branch=master)](https://travis-ci.org/iron/mount)
====

> Mounting middleware for the [Iron](https://github.com/iron/iron) web framework.

## Example

```rust
fn send_hello(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello!")))
}

fn intercept(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Blocked!")))
}

fn main() {
    let mut mount = Mount::new();
    mount.mount("/blocked/", intercept).mount("/", send_hello);

    Iron::new(mount).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
```

## Overview

mount is a part of Iron's [core bundle](https://github.com/iron/core).

- Mount a handler on a sub-path, hiding the old path from that handler.

## Installation

If you're using `Cargo` to manage dependencies, just add mount to the toml:

```toml
[dependencies.mount]

git = "https://github.com/iron/mount.git"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://ironframework.io/doc/mount)

Along with the [online documentation](http://ironframework.io/doc/mount),
you can build a local copy with `cargo doc`.

## [Examples](/examples)

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.
