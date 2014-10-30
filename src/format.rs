use iron::{Request, Response};
use term::{attr, color};
use http::status::NotFound;

use std::default::Default;
use std::from_str::FromStr;

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

        Format::new("@[bold]{method}@@ {uri} @[bold]->@@ @[C]{status}@@ ({response_time})",
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

        // The buffer we will be filling with formatting options.
        let mut result = vec![];

        // String buffers will we use throughout to build relevant Strings.
        let mut string = "".into_string();
        let mut name = "".into_string();

        // Attributes we will set and push into result.
        let mut color = ConstantColor(None);
        let mut attrs = ConstantAttrs(vec![]);

        // The characters of the input string, which we are parsing.
        let mut chars = s.chars();

        loop {
            match chars.next() {
                None => {
                    // No more chars, push our final format unit.
                    result.push(FormatUnit { text: Str(string), color: color, attrs: attrs.clone() });

                    // Done.
                    return Some(Format(result));
                },

                // Parse a thing to print, e.g. {method}, {uri}.
                Some('{') => {
                    result.push(FormatUnit { text: Str(string), color: color, attrs: attrs.clone() });
                    string = "".into_string();

                    loop {
                        match chars.next() {
                            None => return None,

                            Some('}') => {
                                let text = match name.as_slice() {
                                    "method" => Method,
                                    "uri" => URI,
                                    "status" => Status,
                                    "response_time" => ResponseTime,
                                    _ => return None
                                };

                                result.push(FormatUnit { text: text, color: color, attrs: attrs.clone() });
                                name.clear();
                                break;
                            },

                            Some(c) => name.push(c)
                        }
                    }
                },

                // Begin the application of an attribute. The grammar is as follows:
                //
                // `@[attribute]applies to@@`
                //
                // @ begins the attribute, which is then followed by `[attribute]`, which declares
                // what attribute is being applied, then follows the text or {object} that the
                // attribute will be applied to. `@@` then ends the application of the attribute.
                Some('@') => {
                    result.push(FormatUnit { text: Str(string), color: color, attrs: attrs.clone() });
                    string = "".into_string();

                    match chars.next() {
                        // This is actually the end of an attribute application.
                        Some('@') => {
                            color = ConstantColor(None);
                            attrs = ConstantAttrs(vec![]);
                        },

                        // Parse the attribute or style being applied.
                        Some('[') => {
                            loop {
                                match chars.next() {
                                    // Unexpected end of input.
                                    None => { return None; },

                                    // Parse the attribute.
                                    Some(']') => {
                                        for word in name.as_slice().words() {
                                            match word {
                                                "A" => {
                                                    attrs = attrses.pop().unwrap_or(ConstantAttrs(vec![]));
                                                },

                                                "C" => {
                                                    color = colors.pop().unwrap_or(ConstantColor(None));
                                                },

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

                                        name.clear();
                                        break;
                                    },

                                    Some(c) => { name.push(c); }
                                }
                            }
                        }
                        // Unexpected non `@` or `[` after `@`
                        _ => return None,
                    }
                },

                Some(c) => { string.push(c); }
            }
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

