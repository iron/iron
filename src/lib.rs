#![crate_id = "logger"]
#![license = "MIT"]

//! Request logging middleware for Iron

extern crate iron;
extern crate time;
extern crate term;

use std::io::IoResult;

use iron::{Middleware, Alloy, Request, Response};
use iron::middleware::{Status, Continue};

use time::precise_time_ns;

use term::{Terminal, stdout, attr, color};

/// `Middleware` for logging request and response info to the terminal.
/// `Logger` logs the request method, request URI, response status, and response
/// time in the format:
/// ```
/// {method} {uri} -> {status} ({response_time} ms)
/// ```
#[deriving(Clone)]
pub struct Logger {
    entry_time: u64
}

impl Logger {
    /// Create a new `Logger`.
    pub fn new() -> Logger {
        Logger { entry_time: 0u64 }
    }
}

impl Middleware for Logger {
    fn enter(&mut self, _req: &mut Request, _res: &mut Response, _alloy: &mut Alloy) -> Status {
        self.entry_time = precise_time_ns();
        Continue
    }
    fn exit(&mut self, req: &mut Request, res: &mut Response, _al: &mut Alloy) -> Status {
        let ref mut status = res.status;
        let status_color = match status.code() / 100 {
            1 => color::BLUE, // Information
            2 => color::GREEN, // Success
            3 => color::YELLOW, // Redirection
            _ => color::RED, // Error
        };
        let response_time_ms = (precise_time_ns() - self.entry_time) as f64 / 1000000.0;

        // Log to terminal t in the format:
        // {method} {uri} -> {status} ({response_time} ms)
        let log = |mut t: Box<Terminal<Box<Writer + Send>> + Send>| -> IoResult<()> {
            try!(t.attr(attr::Bold));
            try!(write!(t, "{}", req.method));
            try!(t.reset());
            try!(write!(t, " {} ", req.request_uri));
            try!(t.attr(attr::Bold));
            try!(write!(t, "-> "));
            try!(t.reset());
            try!(t.fg(status_color));
            try!(write!(t, "{}", status));
            try!(t.reset());
            try!(writeln!(t, " ({} ms)", response_time_ms));
            Ok(())
        };
        match stdout() {
            Some(terminal) => {
                match log(terminal) {
                    Err(e) => { println!("Error logging to terminal: {}", e); },
                    Ok(_) => ()
                }
            }
            None => { println!("Logger could not open terminal"); }
        }
        Continue
    }
}
