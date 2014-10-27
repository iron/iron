use iron::{Request, Response};
use term::{attr, color};
use http::status::NotFound;

/// A formatting style for the `Logger`, consisting of multiple
/// `FormatUnit`s concatenated into one line.
#[deriving(Clone)]
pub struct Format(pub Vec<FormatUnit>);

impl Format {
    /// Return the default formatting style for the `Logger`:
    ///
    /// ```ignore
    /// {method} {uri} -> {status} ({response_time})
    /// ```
    /// The method is in bold, and the response status is colored blue for 100s,
    /// green for 200s, yellow for 300s, and red for 400s and 500s. For now,
    /// this needs to take `req`/`res` as arguments in order to color the status
    /// appropriately.
    pub fn default() -> Format {
        fn status_color(_req: &Request, res: &Response) -> Option<color::Color> {
            match res.status.as_ref().unwrap_or(&NotFound).code() / 100 {
                1 => Some(color::BLUE), // Information
                2 => Some(color::GREEN), // Success
                3 => Some(color::YELLOW), // Redirection
                4 | 5 => Some(color::RED), // Error
                _ => None
            }
        }
        Format::from_format_string("@[bold]{method}@@ {uri} @[bold]->@@ @[C]{status}@@ ({response_time})",
           &mut vec![FunctionColor(status_color)], &mut vec![]).unwrap()
    }

    /// Create a `Format` from a format string, which can contain the fields
    /// `{method}`, `{uri}`, `{status}`, and `{response_time}`.
    /// Returns `None` if the format string syntax is incorrect.
    pub fn from_format_string(s: &str, colors: &mut Vec<FormatColor>,
                              attrses: &mut Vec<FormatAttrs>) -> Option<Format> {
        let mut result = vec![];
        let mut string = String::from_str("");
        let mut name = String::from_str("");
        let mut chars = s.chars();
        let mut color: FormatColor = ConstantColor(None);
        let mut attrs: FormatAttrs = ConstantAttrs(vec![]);
        loop {
            match chars.next() {
                None => {
                    result.push(FormatUnit {text: Str(string), color: color, attrs: attrs.clone()});
                    return Some(Format(result));
                }
                Some('{') => {
                    result.push(FormatUnit {text: Str(string), color: color, attrs: attrs.clone()});
                    string = String::from_str("");
                    loop {
                        match chars.next() {
                            None => { return None; }
                            Some('}') => {
                                let text = match name.as_slice() {
                                    "method" => Method,
                                    "uri" => URI,
                                    "status" => Status,
                                    "response_time" => ResponseTime,
                                    str => Str(String::from_str(str))
                                };
                                match text {
                                    Str(_) => { return None; }
                                    _ => {
                                        result.push(FormatUnit { text: text, color: color, attrs: attrs.clone() });
                                        name = String::from_str("");
                                        break;
                                    }
                                }
                            }
                            Some(c) => { name.push(c); }
                        }
                    }
                }
                Some('@') => {
                    result.push(FormatUnit {text: Str(string), color: color, attrs: attrs.clone()});
                    string = String::from_str("");
                    match chars.next() {
                        Some('@') => {
                            color = ConstantColor(None);
                            attrs = ConstantAttrs(vec![]);
                        }
                        Some('[') => {
                            loop {
                                match chars.next() {
                                    None => { return None; }
                                    Some(']') => {
                                        for word in name.as_slice().split(' ') {
                                            match word {
                                                "A" => {
                                                    attrs = attrses.remove(0).unwrap_or(ConstantAttrs(vec![]));
                                                }
                                                "C" => {
                                                    color = colors.remove(0).unwrap_or(ConstantColor(None));
                                                }
                                                style => match style_from_name(style) {
                                                    Some(Color(c)) => match color {
                                                        ConstantColor(_) => { color = ConstantColor(Some(c)); }
                                                        FunctionColor(_) => ()
                                                    },
                                                    Some(Attr(a)) => match attrs {
                                                        ConstantAttrs(ref mut v) => { v.push(a); }
                                                        FunctionAttrs(_) => ()
                                                    },
                                                    None => ()
                                                }
                                            }
                                        }
                                        name = String::from_str("");
                                        break;
                                    }
                                    Some(c) => { name.push(c); }
                                }
                            }
                        }
                        _ => { return None; }
                    }
                }
                Some(c) => { string.push(c); }
            }
        }
    }
}

fn style_from_name(name: &str) -> Option<ColorOrAttr> {
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
    fn clone(&self) -> FormatColor {
        match *self {
            ConstantColor(color) => ConstantColor(color),
            FunctionColor(f) => FunctionColor(f)
        }
    }
}

impl Clone for FormatAttrs {
    fn clone(&self) -> FormatAttrs {
        match *self {
            ConstantAttrs(ref attrs) => {
                let mut attrs_clone = vec![];
                for &attr in attrs.iter() {
                    attrs_clone.push(match attr {
                        attr::Bold => attr::Bold,
                        attr::Dim => attr::Dim,
                        attr::Italic(bool) => attr::Italic(bool),
                        attr::Underline(bool) => attr::Underline(bool),
                        attr::Blink => attr::Blink,
                        attr::Standout(bool) => attr::Standout(bool),
                        attr::Reverse => attr::Reverse,
                        attr::Secure => attr::Secure,
                        attr::ForegroundColor(color) => attr::ForegroundColor(color),
                        attr::BackgroundColor(color) => attr::BackgroundColor(color)
                    });
                }
                ConstantAttrs(attrs_clone)
            }
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
