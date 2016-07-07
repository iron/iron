#![deny(missing_docs, warnings)]

//! Request logging middleware for Iron

extern crate iron;
extern crate term;

use iron::{AfterMiddleware, BeforeMiddleware, IronResult, IronError, Request, Response, status};
use iron::typemap::Key;
use term::{StdoutTerminal, color, stdout};

use std::io;
use std::io::Write;
use std::time;

use format::FormatText::{Str, Method, URI, Status, ResponseTime, RemoteAddr};
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
    /// let mut chain = Chain::new(handler);
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
impl Key for StartTime { type Value = time::Instant; }

impl Logger {
    fn initialise(&self, req: &mut Request) {
        req.extensions.insert::<StartTime>(time::Instant::now());
    }

    fn log(&self, req: &mut Request, res: &Response) -> IronResult<()> {
        let entry_time = *req.extensions.get::<StartTime>().unwrap();

        let response_time = entry_time.elapsed();
        let response_time_ms = (response_time.as_secs() * 1000) as f64 + (response_time.subsec_nanos() as f64) / 1000000.0;
        let Format(format) = self.format.clone().unwrap_or_default();

        {
            macro_rules! tryio {
                ($expr:expr) => (match $expr {
                    std::result::Result::Ok(val) => val,
                    std::result::Result::Err(err) => {
                        return Err(IronError::new(err, status::InternalServerError))
                    }
                })
            }

            let render = |text: &FormatText| {
                match *text {
                    Str(ref string) => string.clone(),
                    Method => format!("{}", req.method),
                    URI => format!("{}", req.url),
                    Status => format!("{}", res.status.unwrap()),
                    ResponseTime => format!("{} ms", response_time_ms),
                    RemoteAddr => format!("{}", req.remote_addr)
                }
            };

            let log = |w: &mut LogWriter| -> IronResult<()> {
                for unit in format.iter() {
                    let c = match unit.color {
                        ConstantColor(color) => color,
                        FunctionColor(f) => f(req, res)
                    };
                    let fn_attrs;
                    let ref attrs = match unit.attrs {
                        ConstantAttrs(ref attrs) => attrs,
                        FunctionAttrs(f) => {
                            fn_attrs = f(req, res);
                            &fn_attrs
                        }
                    };
                    tryio!(w.write_item(render(&unit.text), c, attrs));
                }
                tryio!(w.new_line());
                Ok(())
            };

            match stdout() {
                Some(terminal) => {
                    try!(log(&mut TermWriter::new(terminal)));
                }
                None => {
                    try!(log(&mut io::stdout()));
                }
            };
        }

        Ok(())
    }
}

trait LogWriter {
    fn write_item(&mut self, text: String, color: Option<color::Color>, attrs: &Vec<term::Attr>) -> io::Result<()>;
    fn new_line(&mut self) -> io::Result<()>;
}

struct TermWriter {
    term: Box<StdoutTerminal>
}

impl TermWriter {
    fn new(term: Box<StdoutTerminal>) -> TermWriter {
        TermWriter {
            term: term
        }
    }
}

impl LogWriter for TermWriter {
    fn write_item(&mut self, text: String, color: Option<color::Color>, attrs: &Vec<term::Attr>) -> io::Result<()> {
        match color {
            Some(c) => { try!(self.term.fg(c)); }
            None => {},
        }
        for &attr in attrs.iter() {
            try!(self.term.attr(attr));
        }
        try!(write!(self.term, "{}", text));
        try!(self.term.reset());
        Ok(())
    }

    fn new_line(&mut self) -> io::Result<()> {
        try!(writeln!(self.term, ""));
        Ok(())
    }
}

impl<T: Write> LogWriter for T {
    fn write_item(&mut self, text: String, _: Option<color::Color>, _: &Vec<term::Attr>) -> io::Result<()> {
        try!(write!(self, "{}", text));
        Ok(())
    }

    fn new_line(&mut self) -> io::Result<()> {
        try!(writeln!(self, ""));
        Ok(())
    }
}

impl BeforeMiddleware for Logger {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        self.initialise(req);
        Ok(())
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<()> {
        self.initialise(req);
        Err(err)
    }
}

impl AfterMiddleware for Logger {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        try!(self.log(req, &res));
        Ok(res)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        try!(self.log(req, &err.response));
        Err(err)
    }
}
