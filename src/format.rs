//! Formatting helpers for the logger middleware.

use iron::{Request, Response};
use term::{self, color};
use iron::status::NotFound;

use std::default::Default;
use std::str::FromStr;
use std::str::Chars;
use std::vec::IntoIter;
use std::iter::Peekable;

use self::ColorOrAttr::{Color, Attr};
use self::FormatText::{Method, URI, Status, ResponseTime, RemoteAddr};
use self::FormatColor::{ConstantColor, FunctionColor};
use self::FormatAttr::{ConstantAttrs, FunctionAttrs};

/// A formatting style for the `Logger`, consisting of multiple
/// `FormatUnit`s concatenated into one line.
#[derive(Clone)]
pub struct Format(pub Vec<FormatUnit>);

impl Default for Format {
    /// Return the default formatting style for the `Logger`:
    ///
    /// ```ignore
    /// @[bold]{method}@ {uri} @[bold]->@ @[C]{status}@ ({response-time})
    /// // This will be written as: {method} {uri} -> {status} ({response-time})
    /// // with certain style attributes.
    /// ```
    ///
    /// The method is in bold, and the response status is colored blue for 100s,
    /// green for 200s, yellow for 300s, red for 400s, and bright red for 500s.
    fn default() -> Format {
        fn status_color(_req: &Request, res: &Response) -> Option<color::Color> {
            use iron::status::StatusClass::*;

            match res.status.unwrap_or(NotFound).class() {
                Informational   => Some(color::BLUE),
                Success         => Some(color::GREEN),
                Redirection     => Some(color::YELLOW),
                ClientError     => Some(color::RED),
                ServerError     => Some(color::BRIGHT_RED),
                NoClass         => None
            }
        }

        Format::new("@[bold]{method}@ {uri} @[bold]->@ @[C]{status}@ ({response-time})",
           vec![FunctionColor(status_color)], vec![]).unwrap()
    }
}

impl Format {
    // TODO: Document the color/attribute tags.
    /// Create a `Format` from a format string, which can contain the fields
    /// `{method}`, `{uri}`, `{status}`, `{response-time}`, and `{ip-addr}`.
    ///
    /// Returns `None` if the format string syntax is incorrect.
    ///
    /// ---
    ///
    /// Colors and attributes can also be added to the format string within `@` delimiters,
    /// by specifying them in a space-delimited list within square brackets (`[bold italic]`).
    /// They can be made dependent on the request/response by passing `FunctionColor` and
    /// `FunctionAttr`s in as the `colors` and `attrs` vecs; these colors/attributes will
    /// be used sequentially when there is a `[C]` or `[A]` marker, respectively (`[bold C]`).
    ///
    /// For example: `@[bold C]{status}@` will be formatted based upon
    /// the first FormatColor constant or function in the `colors` vector,
    /// yielding a bold and colored response status.
    ///
    /// Available colors are:
    ///
    /// - `black`
    /// - `blue`
    /// - `brightblack`
    /// - `brightblue`
    /// - `brightcyan`
    /// - `brightgreen`
    /// - `brightmagenta`
    /// - `brightred`
    /// - `brightwhite`
    /// - `brightyellow`
    /// - `cyan`
    /// - `green`
    /// - `magenta`
    /// - `red`
    /// - `white`
    /// - `yellow`
    ///
    /// Available attributes are:
    ///
    /// - `bold`
    /// - `dim`
    /// - `italic`
    /// - `underline`
    /// - `blink`
    /// - `standout`
    /// - `reverse`
    /// - `secure`
    pub fn new(s: &str, colors: Vec<FormatColor>, attrs: Vec<FormatAttr>)
            -> Option<Format> {

        let parser = FormatParser::new(s.chars().peekable(),
                                       colors.into_iter(),
                                       attrs.into_iter());

        let mut results = Vec::new();

        for unit in parser {
            match unit {
                Some(unit) => results.push(unit),
                None => return None
            }
        }

        Some(Format(results))
    }
}

struct FormatParser<'a> {
    // The characters of the format string.
    chars: Peekable<Chars<'a>>,

    // Passed-in FormatColors
    colors: IntoIter<FormatColor>,

    // Passed-in FormatAttr
    attrs: IntoIter<FormatAttr>,

    // A reusable buffer for parsing style attributes.
    object_buffer: String,

    // A queue of waiting format units to avoid full-on
    // state-machine parsing.
    waitqueue: Vec<FormatUnit>,

    finished: bool
}

impl<'a> FormatParser<'a> {
    fn new(chars: Peekable<Chars>, colors: IntoIter<FormatColor>,
           attrs: IntoIter<FormatAttr>) -> FormatParser {
        FormatParser {
            chars: chars,
            colors: colors,
            attrs: attrs,

            // No attributes are longer than 14 characters, so we can avoid reallocating.
            object_buffer: String::with_capacity(14),

            waitqueue: vec![],
            finished: false
        }
    }
}

// Some(None) means there was a parse error and this FormatParser should be abandoned.
impl<'a> Iterator for FormatParser<'a> {
    type Item = Option<FormatUnit>;

    fn next(&mut self) -> Option<Option<FormatUnit>> {
        // If the parser has been cancelled or errored for some reason.
        if self.finished { return None }

        if self.waitqueue.len() != 0 {
            return Some(Some(self.waitqueue.remove(0)));
        }

        // Try to parse a new FormatUnit.
        match self.chars.next() {
            // Parse a recognized object.
            //
            // The allowed forms are:
            //   - {method}
            //   - {uri}
            //   - {status}
            //   - {response-time}
            //   - {ip-addr}
            Some('{') => {
                self.object_buffer.clear();

                let mut chr = self.chars.next();
                while chr != None {
                    match chr.unwrap() {
                        // Finished parsing, parse buffer.
                        '}' => break,
                        c => self.object_buffer.push(c.clone())
                    }

                    chr = self.chars.next();
                }

                let text = match self.object_buffer.as_ref() {
                    "method" => Method,
                    "uri" => URI,
                    "status" => Status,
                    "response-time" => ResponseTime,
                    "ip-addr" => RemoteAddr,
                    _ => {
                        // Error, so mark as finished.
                        self.finished = true;
                        return Some(None);
                    }
                };

                Some(Some(FormatUnit {
                    text: text,
                    color: ConstantColor(None),
                    attrs: ConstantAttrs(vec![])
                }))
            },

            // Parse an attribute and the thing it applies to.
            //
            // The form is:
            //   - @[attributes]target@
            Some('@') => {
                match self.chars.next() {
                    // Parse attributes
                    Some('[') => {
                        let mut buffer = String::new();

                        loop {
                            match self.chars.next() {
                                // Finished parsing into buffer.
                                Some(']') => break,
                                Some(c) => buffer.push(c),
                                // Error, so mark as finished.
                                None => {
                                    self.finished = true;
                                    return Some(None);
                                }
                            }
                        }

                        let mut attrs = ConstantAttrs(vec![]);
                        let mut color = ConstantColor(None);

                        // Collect the attributes into attrs and color, for use as properties
                        // of a FormatUnit.
                        for word in buffer.split(|c: char| c.is_whitespace()) {
                            match word {
                                "A" => attrs = self.attrs.next().unwrap_or(ConstantAttrs(vec![])),

                                "C" => color = self.colors.next().unwrap_or(ConstantColor(None)),

                                style => match style.parse() {
                                    Ok(Color(c)) => match color {
                                        ConstantColor(_) => { color = ConstantColor(Some(c)); },
                                        _ => {}
                                    },

                                    Ok(Attr(a)) => match attrs {
                                        ConstantAttrs(ref mut v) => { v.push(a); },
                                        _ => {}
                                    },

                                    _ => {}
                                }
                            }
                        }

                        // Nested attributes are not supported.
                        if self.chars.peek() == Some(&'@') {
                            self.finished = true;
                            return Some(None);
                        }

                        // Now we have the parsed attributes, so we can parse what they apply to.
                        let mut apply_to = vec![];

                        loop {
                            match self.next() {
                                Some(Some(unit)) => apply_to.push(unit),
                                _ => {
                                    self.finished = true;
                                    return Some(None);
                                }
                            };

                            if self.chars.peek() == Some(&'@') {
                                // Jump over the closing '@'
                                self.chars.next();
                                break;
                            }
                        }

                        self.waitqueue.extend(apply_to.into_iter().map(|unit| {
                            FormatUnit {
                                text: unit.text,
                                color: color.clone(),
                                attrs: attrs.clone()
                            }
                        }));

                        self.next()
                    },

                    _ => {
                        // Error, so mark as finished.
                        self.finished = true;
                        return Some(None);
                    }
                }
            },

            // Parse a regular string part of the format string.
            Some(c) => {
                let mut buffer = String::new();
                buffer.push(c);

                loop {
                    match self.chars.peek() {
                        // Done parsing.
                        Some(&'@') | Some(&'{') | None => {
                            return Some(Some(FormatUnit {
                                text: FormatText::Str(buffer),
                                color: ConstantColor(None),
                                attrs: ConstantAttrs(vec![])
                            }))
                        },

                        Some(_) => {
                            buffer.push(self.chars.next().unwrap())
                        }
                    }
                }
            },

            // Reached end of the format string.
            None => None
        }
    }
}

impl FromStr for ColorOrAttr {
    type Err = ();

    fn from_str(name: &str) -> Result<ColorOrAttr, ()> {
        match name {
            "black" => Ok(Color(color::BLACK)),
            "blue" => Ok(Color(color::BLUE)),
            "brightblack" => Ok(Color(color::BRIGHT_BLACK)),
            "brightblue" => Ok(Color(color::BRIGHT_BLUE)),
            "brightcyan" => Ok(Color(color::BRIGHT_CYAN)),
            "brightgreen" => Ok(Color(color::BRIGHT_GREEN)),
            "brightmagenta" => Ok(Color(color::BRIGHT_MAGENTA)),
            "brightred" => Ok(Color(color::BRIGHT_RED)),
            "brightwhite" => Ok(Color(color::BRIGHT_WHITE)),
            "brightyellow" => Ok(Color(color::BRIGHT_YELLOW)),
            "cyan" => Ok(Color(color::CYAN)),
            "green" => Ok(Color(color::GREEN)),
            "magenta" => Ok(Color(color::MAGENTA)),
            "red" => Ok(Color(color::RED)),
            "white" => Ok(Color(color::WHITE)),
            "yellow" => Ok(Color(color::YELLOW)),

            "bold" => Ok(Attr(term::Attr::Bold)),
            "dim" => Ok(Attr(term::Attr::Dim)),
            "italic" => Ok(Attr(term::Attr::Italic(true))),
            "underline" => Ok(Attr(term::Attr::Underline(true))),
            "blink" => Ok(Attr(term::Attr::Blink)),
            "standout" => Ok(Attr(term::Attr::Standout(true))),
            "reverse" => Ok(Attr(term::Attr::Reverse)),
            "secure" => Ok(Attr(term::Attr::Secure)),

            _ => Err(())
        }
    }
}

enum ColorOrAttr {
    Color(color::Color),
    Attr(term::Attr)
}

/// A representation of color in a `FormatUnit`.
#[derive(Copy)]
pub enum FormatColor {
    /// A constant color
    ConstantColor(Option<color::Color>),
    /// A variable color, dependent on the request/response
    ///
    /// This can be used to change the color depending on response status, &c.
    FunctionColor(fn(&Request, &Response) -> Option<color::Color>)
}

/// A representation of attributes in a `FormatUnit`.
pub enum FormatAttr {
    /// A constant attribute
    ConstantAttrs(Vec<term::Attr>),
    /// A variable attribute, dependent on the request/response
    ///
    /// This can be used to change the attribute depending on response status, &c.
    FunctionAttrs(fn(&Request, &Response) -> Vec<term::Attr>)
}

impl Clone for FormatColor {
    fn clone(&self) -> FormatColor { *self }
}

impl Clone for FormatAttr {
    fn clone(&self) -> FormatAttr {
        match *self {
            ConstantAttrs(ref attrs) => ConstantAttrs(attrs.iter().map(|&attr| attr).collect()),
            FunctionAttrs(f) => FunctionAttrs(f)
        }
    }
}

/// A string of text to be logged. This is either one of the data
/// fields supported by the `Logger`, or a custom `String`.
#[derive(Clone)]
#[doc(hidden)]
pub enum FormatText {
    Str(String),
    Method,
    URI,
    Status,
    ResponseTime,
    RemoteAddr
}

/// A `FormatText` with associated style information.
#[derive(Clone)]
#[doc(hidden)]
pub struct FormatUnit {
    pub text: FormatText,
    pub color: FormatColor,
    pub attrs: FormatAttr
}
