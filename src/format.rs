use iron::{Request, Response};
use term::{attr, color};

pub enum FormatText {
    Str(&'static str),
    Method,
    URI,
    Status,
    ResponseTime
}

pub struct FormatUnit {
    pub text: FormatText,
    pub color: Option<color::Color>,
    pub attrs: Vec<attr::Attr>
}

pub struct Format(pub Vec<FormatUnit>);

impl Format {
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
