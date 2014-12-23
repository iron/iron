#![deny(missing_docs, warnings)]

//! Request logging middleware for Iron

extern crate iron;
extern crate time;
extern crate term;
extern crate typemap;

use iron::{AfterMiddleware, BeforeMiddleware, IronResult, IronError, Request, Response};
use iron::typemap::Assoc;
use term::{Terminal, WriterWrapper, stdout};

use std::io::IoResult;
use std::error::Error;

use format::FormatText::{Str, Method, URI, Status, ResponseTime};
use format::FormatColor::{ConstantColor, FunctionColor};
use format::FormatAttr::{ConstantAttrs, FunctionAttrs};
use format::{Format, FormatText};

pub mod format;

/// `Middleware` for logging request and response info to the terminal.
pub struct Logger {
    format: Option<Format>
}

impl Logger {
    /// Create a pair of `Logger` middlewares with the specified `format`. If a `None` is passed in, uses the default format:
    ///
    /// ```ignore
    /// {method} {uri} -> {status} ({response_time} ms)
    /// ```
    ///
    /// While the returned value can be passed straight to `Chain::link`, consider making the logger `BeforeMiddleware`
    /// the first in your chain and the logger `AfterMiddleware` the last by doing something like this:
    ///
    /// ```ignore
    /// let mut chain = ChainBuilder::new(handler);
    /// let (logger_before, logger_after) = Logger::new(None);
    /// chain.link_before(logger_before);
    /// // link other middlewares here...
    /// chain.link_after(logger_after);
    /// ```
    pub fn new(format: Option<Format>) -> (Logger, Logger) {
        (Logger { format: format.clone() }, Logger { format: format })
    }
}

struct StartTime;
impl Assoc<u64> for StartTime {}

impl BeforeMiddleware for Logger {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<StartTime, u64>(time::precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for Logger {
    fn after(&self, req: &mut Request, res: &mut Response) -> IronResult<()> {
        let exit_time = time::precise_time_ns();
        let entry_time = *req.extensions.get::<StartTime, u64>().unwrap();

        let response_time_ms = (exit_time - entry_time) as f64 / 1000000.0;
        let Format(format) = self.format.clone().unwrap_or_default();

        let render = |text: &FormatText| {
            match *text {
                Str(ref string) => string.clone(),
                Method => format!("{}", req.method),
                URI => format!("{}", req.url),
                Status => format!("{}", res.status),
                ResponseTime => format!("{} ms", response_time_ms)
            }
        };

        let log = |mut t: Box<Terminal<WriterWrapper> + Send>| -> IoResult<()> {
            for unit in format.iter() {
                match unit.color {
                    ConstantColor(Some(color)) => { try!(t.fg(color)); }
                    ConstantColor(None) => (),
                    FunctionColor(f) => match f(req, res) {
                        Some(color) => { try!(t.fg(color)); }
                        None => ()
                    }
                }
                match unit.attrs {
                    ConstantAttrs(ref attrs) => {
                        for &attr in attrs.iter() {
                            try!(t.attr(attr));
                        }
                    }
                    FunctionAttrs(f) => {
                        for &attr in f(req, res).iter() {
                            try!(t.attr(attr));
                        }
                    }
                }
                try!(write!(t, "{}", render(&unit.text)));
                try!(t.reset());
            }
            try!(writeln!(t, ""));
            Ok(())
        };

        match stdout() {
            Some(terminal) => {
                try!(log(terminal));
            }
            None => { return Err(box CouldNotOpenTerminal as IronError) }
        };

        Ok(())
    }
}

/// Error returned when logger cannout access stdout.
#[deriving(Show, Copy)]
pub struct CouldNotOpenTerminal;

impl Error for CouldNotOpenTerminal {
    fn description(&self) -> &str {
        "Could Not Open Terminal"
    }

    fn detail(&self) -> Option<String> {
        Some("Logger could not open stdout as a terminal.".into_string())
    }
}

