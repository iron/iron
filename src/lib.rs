#![deny(missing_docs, warnings)]

//! Request logging middleware for Iron

extern crate iron;
#[macro_use] extern crate log;
extern crate time;

use iron::{AfterMiddleware, BeforeMiddleware, IronResult, IronError, Request, Response};
use iron::typemap::Key;

use format::FormatText::{Str, Method, URI, Status, ResponseTime};
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
impl Key for StartTime { type Value = u64; }

impl Logger {
    fn initialise(&self, req: &mut Request) {
        req.extensions.insert::<StartTime>(time::precise_time_ns());
    }

    fn log(&self, req: &mut Request, res: &Response) -> IronResult<()> {
        let exit_time = time::precise_time_ns();
        let entry_time = *req.extensions.get::<StartTime>().unwrap();

        let response_time_ms = (exit_time - entry_time) as f64 / 1000000.0;
        let Format(format) = self.format.clone().unwrap_or_default();

        {
            let render = |text: &FormatText| {
                match *text {
                    Str(ref string) => string.clone(),
                    Method => format!("{}", req.method),
                    URI => format!("{}", req.url),
                    Status => format!("{}", res.status.unwrap()),
                    ResponseTime => format!("{} ms", response_time_ms)
                }
            };

            let lg = format.iter().map(|unit| render(&unit.text)).collect::<Vec<String>>().join("");
            info!(target: "cms::access", "{}", lg);
        }

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
