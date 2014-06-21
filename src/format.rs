use iron::{Request, Response};
use term::{attr, color};

/// A formatting style for the `Logger`, consisting of multiple
/// `FormatUnit`s concatenated into one line.
#[deriving(Clone)]
pub struct Format(pub Vec<FormatUnit>);

impl Format {
    /// Return the default formatting style for the `Logger`:
    ///
    /// ```
    /// {method} {uri} -> {status} ({response_time})
    /// ```
    /// The method is in bold, and the response status is colored blue for 100s,
    /// green for 200s, yellow for 300s, and red for 400s and 500s. For now,
    /// this needs to take `req`/`res` as arguments in order to color the status
    /// appropriately.
    pub fn default() -> Format {
        fn status_color(_req: &Request, res: &Response) -> Option<color::Color> {
            match res.status.code() / 100 {
                1 => Some(color::BLUE), // Information
                2 => Some(color::GREEN), // Success
                3 => Some(color::YELLOW), // Redirection
                4 | 5 => Some(color::RED), // Error
                _ => None
            }
        }
        Format::from_format_string("{method} {uri} -> @[]{status}@@ ({response_time})",
           &mut vec![FunctionColor(status_color)]).unwrap()
    }

    /// Create a `Format` from a format string, which can contain the fields
    /// `{method}`, `{uri}`, `{status}`, and `{response_time}`.
    /// Returns `None` if the format string syntax is incorrect.
    pub fn from_format_string(s: &str, colors: &mut Vec<FormatColor>) -> Option<Format> {
        let mut result = vec![];
        let mut string = String::from_str("");
        let mut name = String::from_str("");
        let mut chars = s.chars();
        let mut color: FormatColor = ConstantColor(None);
        loop {
            match chars.next() {
                None => {
                    result.push(FormatUnit {text: Str(string), color: color, attrs: vec![]});
                    return Some(Format(result));
                }
                Some('{') => {
                    result.push(FormatUnit {text: Str(string), color: color, attrs: vec![]});
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
                                        result.push(FormatUnit { text: text, color: color, attrs: vec![] });
                                        name = String::from_str("");
                                        break;
                                    }
                                }
                            }
                            Some(c) => { name.push_char(c); }
                        }
                    }
                }
                Some('@') => {
                    result.push(FormatUnit {text: Str(string), color: color, attrs: vec![]});
                    string = String::from_str("");
                    match chars.next() {
                        Some('@') => { color = ConstantColor(None); }
                        Some('[') => {
                            loop {
                                match chars.next() {
                                    None => { return None; }
                                    Some(']') => {
                                        color = match name.as_slice() {
                                            "red" => ConstantColor(Some(color::RED)),
                                            "blue" => ConstantColor(Some(color::BLUE)),
                                            "yellow" => ConstantColor(Some(color::YELLOW)),
                                            "green" => ConstantColor(Some(color::GREEN)),
                                            "" => colors.shift().unwrap_or(ConstantColor(None)),
                                            _ => ConstantColor(None)
                                        };
                                        name = String::from_str("");
                                        break;
                                    }
                                    Some(c) => { name.push_char(c); }
                                }
                            }
                        }
                        _ => { return None; }
                    }
                }
                Some(c) => { string.push_char(c); }
            }
        }
    }
}

/// A representation of color in a `FormatUnit`.
pub enum FormatColor {
    ConstantColor(Option<color::Color>),
    FunctionColor(fn(&Request, &Response) -> Option<color::Color>)
}

impl Clone for FormatColor {
    fn clone(&self) -> FormatColor {
        match *self {
            ConstantColor(color) => ConstantColor(color),
            FunctionColor(f) => FunctionColor(f)
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
pub struct FormatUnit {
    pub text: FormatText,
    pub color: FormatColor,
    pub attrs: Vec<attr::Attr>
}

impl Clone for FormatUnit {
    fn clone(&self) -> FormatUnit {
        let mut attrs = vec![];
        for &attr in self.attrs.iter() {
            attrs.push(match attr {
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
        FormatUnit {
            text: self.text.clone(),
            color: self.color.clone(),
            attrs: attrs
        }
    }
}
