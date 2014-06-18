#![crate_id = "logger"]
#![license = "MIT"]

//! Request logging middleware for Iron

extern crate iron;
extern crate time;
extern crate term;

use iron::{Middleware, Alloy, Request, Response};
use iron::middleware::{Status, Continue};
use time::precise_time_ns;
use term::{Terminal, stdout};

use std::io::IoResult;

use format::{Format, FormatText, Str, Method, URI, Status, ResponseTime};

pub mod format;

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
        let response_time_ms = (precise_time_ns() - self.entry_time) as f64 / 1000000.0;
        let Format(format) = Format::default(req, res);

        let render = |text: FormatText| {
            match text {
                Str(str) => String::from_str(str),
                Method => format!("{}", req.method),
                URI => format!("{}", req.request_uri),
                Status => format!("{}", res.status),
                ResponseTime => format!("{} ms", response_time_ms)
            }
        };
        let log = |mut t: Box<Terminal<Box<Writer + Send>> + Send>| -> IoResult<()> {
            for unit in format.iter() {
                match unit.color {
                    Some(color) => { try!(t.fg(color)); }
                    None => ()
                }
                for &attr in unit.attrs.iter() {
                    try!(t.attr(attr));
                }
                try!(write!(t, "{}", render(unit.text)));
                try!(t.reset());
            }
            try!(writeln!(t, ""));
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
