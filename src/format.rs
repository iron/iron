use iron::{Request, Response};
use term::{attr, color};

/// A string of text to be logged. This is either one of the data
/// fields supported by the `Logger`, or a custom `&str`.
pub enum FormatText {
    Str(&'static str),
    Method,
    URI,
    Status,
    ResponseTime
}

/// A `FormatText` with associated style information.
pub struct FormatUnit {
    pub text: FormatText,
    pub color: Option<color::Color>,
    pub attrs: Vec<attr::Attr>
}

/// A formatting style for the `Logger`, consisting of multiple
/// `FormatUnit`s concatenated into one line.
pub struct Format(pub Vec<FormatUnit>);

impl Format {
    /// Returns the default formatting style for the `Logger`:
    ///
    /// ```
    /// {method} {uri} -> {status} ({response_time} ms)
    /// ```
    /// The method is in bold, and the response status is colored blue for 100s,
    /// green for 200s, yellow for 300s, and red for 400s and 500s. For now,
    /// this needs to take `req`/`res` as arguments in order to color the status
    /// appropriately.
    pub fn default(_req: &Request, res: &Response) -> Format {
        let status_color = match res.status.code() / 100 {
            1 => color::BLUE, // Information
            2 => color::GREEN, // Success
            3 => color::YELLOW, // Redirection
            _ => color::RED, // Error
        };
        Format(vec![
            FormatUnit { text: Method, color: None, attrs: vec![attr::Bold] },
            FormatUnit { text: Str(" "), color: None, attrs: vec![] },
            FormatUnit { text: URI, color: None, attrs: vec![] },
            FormatUnit { text: Str(" -> "), color: None, attrs: vec![attr::Bold] },
            FormatUnit { text: Status, color: Some(status_color), attrs: vec![] },
            FormatUnit { text: Str(" ("), color: None, attrs: vec![] },
            FormatUnit { text: ResponseTime, color: None, attrs: vec![] },
            FormatUnit { text: Str(")"), color: None, attrs: vec![] }
        ])
    }
}
