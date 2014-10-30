use iron::{Request, Response};
use term::{attr, color};
use iron::status::NotFound;

use std::default::Default;
use std::from_str::FromStr;
use std::str::Chars;
use std::vec::MoveItems;
use std::iter::Peekable;

/// A formatting style for the `Logger`, consisting of multiple
/// `FormatUnit`s concatenated into one line.
#[deriving(Clone)]
pub struct Format(pub Vec<FormatUnit>);

impl Default for Format {
    /// Return the default formatting style for the `Logger`:
    ///
    /// ```ignore
    /// {method} {uri} -> {status} ({response_time})
    /// ```
    ///
    /// The method is in bold, and the response status is colored blue for 100s,
    /// green for 200s, yellow for 300s, and red for 400s and 500s. For now,
    /// this needs to take `req`/`res` as arguments in order to color the status
    /// appropriately.
    fn default() -> Format {
        fn status_color(_req: &Request, res: &Response) -> Option<color::Color> {
            match res.status.as_ref().unwrap_or(&NotFound).code() / 100 {
                1 => Some(color::BLUE), // Information
                2 => Some(color::GREEN), // Success
                3 => Some(color::YELLOW), // Redirection
                4 | 5 => Some(color::RED), // Error
                _ => None
            }
        }

        Format::new("@[bold]{method}@ {uri} @[bold]->@ @[C]{status}@ ({response-time})",
           vec![FunctionColor(status_color)], vec![]).unwrap()
    }
}

impl Format {
    /// Create a `Format` from a format string, which can contain the fields
    /// `{method}`, `{uri}`, `{status}`, and `{response_time}`.
    /// Returns `None` if the format string syntax is incorrect.
    pub fn new(s: &str, mut colors: Vec<FormatColor>,
               mut attrses: Vec<FormatAttrs>) -> Option<Format> {

        // We use these as stacks, but in the wrong direction.
        attrses.reverse();
        colors.reverse();

        let mut parser = FormatParser::new(s.chars().peekable(), colors.into_iter(), attrses.into_iter());

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
    colors: MoveItems<FormatColor>,

    // Passed-in FormatAttrs
    attrs: MoveItems<FormatAttrs>,

    // A reusable buffer for parsing style attributes.
    object_buffer: String,

    // Are we done?
    finished: bool,

    // A queue of waiting format units to avoid full-on
    // state-machine parsing.
    waitqueue: Vec<FormatUnit>
}

impl<'a> FormatParser<'a> {
    fn new(chars: Peekable<char, Chars>, colors: MoveItems<FormatColor>, attrs: MoveItems<FormatAttrs>) -> FormatParser {
        FormatParser {
            chars: chars,
            colors: colors,
            attrs: attrs,

            // No attributes are longer than 14 characters, so we can
            // avoid reallocating.
            object_buffer: String::with_capacity(14),

            finished: false,
            waitqueue: vec![]
        }
    }
}

// Some(None) means there was a parse error and this FormatParser
// should be abandoned.
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

                loop {
                    match self.chars.next() {
                        // Finished parsing, parse buffer.
                        Some('}') => break,

                        Some(c) => self.object_buffer.push(c),

                        None => break
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

                                // Push into buffer.
                                Some(c) => buffer.push(c),

                                None => {
                                    // Error, so mark as finished.
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

                                style => match from_str(style) {
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
                                text: Str(buffer),
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
pub enum FormatColor {
    ConstantColor(Option<color::Color>),
    FunctionColor(fn(&Request, &Response) -> Option<color::Color>)
}

/// A representation of attributes in a `FormatUnit`.
pub enum FormatAttrs {
    ConstantAttrs(Vec<attr::Attr>),
    FunctionAttrs(fn(&Request, &Response) -> Vec<attr::Attr>)
}

impl Clone for FormatColor {
    fn clone(&self) -> FormatColor { *self }
}

impl Clone for FormatAttrs {
    fn clone(&self) -> FormatAttrs {
        match *self {
            ConstantAttrs(ref attrs) => ConstantAttrs(attrs.iter().map(|&attr| attr).collect()),
            FunctionAttrs(f) => FunctionAttrs(f)
        }
    }
}

/// A string of text to be logged. This is either one of the data
/// fields supported by the `Logger`, or a custom `&str`.
#[deriving(Clone)]
pub enum FormatText {
    Str(String),
    Method,
    URI,
    Status,
    ResponseTime
}

/// A `FormatText` with associated style information.
#[deriving(Clone)]
pub struct FormatUnit {
    pub text: FormatText,
    pub color: FormatColor,
    pub attrs: FormatAttrs
}

