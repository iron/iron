Iron [![Build Status](https://secure.travis-ci.org/iron/iron.png?branch=master)](https://travis-ci.org/iron/iron)
====

> Express inspired, rapid, scalable, concurrent and safe server development

Iron is a high level web framework built in and for Rust. Iron does not come
bundled with any middleware, which Iron calls Ingots - instead, Iron is a
robust and efficient framework for plugging in middleware.

After spawning, handling a single request through Iron’s middleware stack
with a single no-op middleware takes only _300 nanoseconds_.

## Installation

```bash
./configure   # Gets all dependencies and builds them
make lib      # Build Iron itself -- you can stop here if you just want the library
make test     # Build and run tests
make examples # Build the examples
make doc      # Build documentation using rustdoc
```

## Get Help

One of us (@reem, @zzmp, @theptrk, @mcreinhard) is usually on `#iron` on the
mozilla irc. Come say hi and ask any questions you might have. We are also
usually on `#rust`.

## Overview

Iron aims to fill a void in the Rust web stack - a high level framework that is
*extensible* and makes organizing complex server code easy.

Whereas other web frameworks have focused mostly on creating an easy-to-use
routing system, Iron focuses on providing a clean API for creating
Ingots/middleware and integrating them in Iron servers.

In fact, Routing is middleware in Iron, as is Mounting, Body Parsing, and most
other features. This allows for insanely flexible setups and allows almost all
of Iron’s features to be swappable - you can even change the middleware
resolution algorithm by swapping in your own Furnace.

