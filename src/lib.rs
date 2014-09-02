#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/logger")]
#![crate_name = "logger"]
#![license = "MIT"]

//! Request logging middleware for Iron

extern crate iron;
extern crate http;
extern crate time;
extern crate term;
extern crate typemap;

use iron::{AfterMiddleware, BeforeMiddleware, IronResult, Request, Response};
use typemap::Assoc;
use time::precise_time_ns;
use term::{Terminal, WriterWrapper, stdout};

use std::io::IoResult;

use format::{Format, FormatText, Str, Method, URI, Status, ResponseTime,
             ConstantColor, FunctionColor, ConstantAttrs, FunctionAttrs};

pub mod format;

/// `Middleware` for logging request and response info to the terminal.
pub struct Logger {
    format: Option<Format>
}

pub struct LoggerBeforeMiddleware;

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
    pub fn new(format: Option<Format>) -> (LoggerBeforeMiddleware, Logger) {
        (LoggerBeforeMiddleware, Logger { format: format })
    }
}

struct StartTime;
impl Assoc<u64> for StartTime {}

impl BeforeMiddleware for LoggerBeforeMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<StartTime, u64>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for Logger {
    fn after(&self, req: &mut Request, res: &mut Response) -> IronResult<()> {
        let entry_time = *req.extensions.find::<StartTime, u64>().unwrap();
        let response_time_ms = (precise_time_ns() - entry_time) as f64 / 1000000.0;
        let Format(format) = self.format.clone().unwrap_or(Format::default());

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
                match log(terminal) {
                    Err(e) => { println!("Error logging to terminal: {}", e); },
                    Ok(_) => ()
                }
            }
            None => { println!("Logger could not open terminal"); }
        }
        Ok(())
    }
}
