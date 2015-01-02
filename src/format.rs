//! Formatting helpers for the logger middleware.

use iron::{Request, Response};
use term::{attr, color};
use iron::status::NotFound;

use std::default::Default;
use std::str::FromStr;
use std::str::Chars;
use std::vec::IntoIter;
use std::iter::Peekable;

use self::ColorOrAttr::{Color, Attr};
use self::FormatText::{Method, URI, Status, ResponseTime};
use self::FormatColor::{ConstantColor, FunctionColor};
use self::FormatAttr::{ConstantAttrs, FunctionAttrs};

/// A formatting style for the `Logger`, consisting of multiple
/// `FormatUnit`s concatenated into one line.
#[deriving(Clone)]
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
            match *res.status.as_ref().unwrap_or(&NotFound) as u16 / 100 {
                1 => Some(color::BLUE), // Information
                2 => Some(color::GREEN), // Success
                3 => Some(color::YELLOW), // Redirection
                4 => Some(color::RED), // Client Error
                5 => Some(color::BRIGHT_RED), // Internal Error
                _ => None
            }
        }

        Format::new("@[bold]{method}@ {uri} @[bold]->@ @[C]{status}@ ({response-time})",
           vec![FunctionColor(status_color)], vec![]).unwrap()
    }
}

impl Format {
    // TODO: Document the color/attribute tags.
    /// Create a `Format` from a format string, which can contain the fields
    /// `{method}`, `{uri}`, `{status}`, and `{response-time}`.
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

        let mut parser = FormatParser::new(s.chars().peekable(),
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
    chars: Peekable<char, Chars<'a>>,

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
    fn new(chars: Peekable<char, Chars>, colors: IntoIter<FormatColor>,
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
impl<'a> Iterator<Option<FormatUnit>> for FormatParser<'a> {
    fn next(&mut self) -> Option<Option<FormatUnit>> {
        // If the parser has been cancelled or errored for some reason.
        if self.finished { return None }

        if self.waitqueue.len() != 0 {
            return Some(self.waitqueue.remove(0));
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
            Some('{') => {
                self.object_buffer.clear();

                for chr in self.chars {
                    match chr {
                        // Finished parsing, parse buffer.
                        '}' => break,
                        c => self.object_buffer.push(c)
                    }
                }

                let text = match self.object_buffer.as_slice() {
                    "method" => Method,
                    "uri" => URI,
                    "status" => Status,
                    "response-time" => ResponseTime,
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
                        for word in buffer.as_slice().words() {
                            match word {
                                "A" => attrs = self.attrs.next().unwrap_or(ConstantAttrs(vec![])),

                                "C" => color = self.colors.next().unwrap_or(ConstantColor(None)),

                                style => match style.parse() {
                                    Some(Color(c)) => match color {
                                        ConstantColor(_) => { color = ConstantColor(Some(c)); },
                                        _ => {}
                                    },

                                    Some(Attr(a)) => match attrs {
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
    fn from_str(name: &str) -> Option<ColorOrAttr> {
        match name {
            "black" => Some(Color(color::BLACK)),
            "blue" => Some(Color(color::BLUE)),
            "brightblack" => Some(Color(color::BRIGHT_BLACK)),
            "brightblue" => Some(Color(color::BRIGHT_BLUE)),
            "brightcyan" => Some(Color(color::BRIGHT_CYAN)),
            "brightgreen" => Some(Color(color::BRIGHT_GREEN)),
            "brightmagenta" => Some(Color(color::BRIGHT_MAGENTA)),
            "brightred" => Some(Color(color::BRIGHT_RED)),
            "brightwhite" => Some(Color(color::BRIGHT_WHITE)),
            "brightyellow" => Some(Color(color::BRIGHT_YELLOW)),
            "cyan" => Some(Color(color::CYAN)),
            "green" => Some(Color(color::GREEN)),
            "magenta" => Some(Color(color::MAGENTA)),
            "red" => Some(Color(color::RED)),
            "white" => Some(Color(color::WHITE)),
            "yellow" => Some(Color(color::YELLOW)),

            "bold" => Some(Attr(attr::Bold)),
            "dim" => Some(Attr(attr::Dim)),
            "italic" => Some(Attr(attr::Italic(true))),
            "underline" => Some(Attr(attr::Underline(true))),
            "blink" => Some(Attr(attr::Blink)),
            "standout" => Some(Attr(attr::Standout(true))),
            "reverse" => Some(Attr(attr::Reverse)),
            "secure" => Some(Attr(attr::Secure)),

            _ => None
        }
    }
}

enum ColorOrAttr {
    Color(color::Color),
    Attr(attr::Attr)
}

/// A representation of color in a `FormatUnit`.
#[deriving(Copy)]
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
    ConstantAttrs(Vec<attr::Attr>),
    /// A variable attribute, dependent on the request/response
    ///
    /// This can be used to change the attribute depending on response status, &c.
    FunctionAttrs(fn(&Request, &Response) -> Vec<attr::Attr>)
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
#[deriving(Clone)]
#[doc(hidden)]
pub enum FormatText {
    Str(String),
    Method,
    URI,
    Status,
    ResponseTime
}

/// A `FormatText` with associated style information.
#[deriving(Clone)]
#[doc(hidden)]
pub struct FormatUnit {
    pub text: FormatText,
    pub color: FormatColor,
    pub attrs: FormatAttr
}

