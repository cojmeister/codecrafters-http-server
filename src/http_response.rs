use std::fmt::Display;

#[derive(PartialEq, Eq, Debug)]
pub enum ContentType {
    TextPlain,
    ApplicationOctetStream,
    None
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ContentType::TextPlain => { "text/plain".to_string() }
            ContentType::ApplicationOctetStream => { "application/octet-stream".to_string() }
            ContentType::None => { "".to_string() }
        };
        write!(f, "{}", str)
    }
}


pub struct HttpResponse {
    code: usize,
    message: String,
    content_type: ContentType,
    content_length: usize,
    content: String
}

impl HttpResponse {
    pub fn make_200() -> HttpResponse {
        HttpResponse {
            code: 200,
            message: "OK".to_string(),
            content_type: ContentType::None,
            content_length: 0,
            content: "".to_string()
        }
    }

    pub fn make_404() -> HttpResponse {
        HttpResponse {
            code: 404,
            message: "Not Found".to_string(),
            content_type: ContentType::None,
            content_length: 0,
            content: "".to_string()
        }
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut base = format!("HTTP/1.1 {} {}", self.code, self.message);
        if self.content_type != ContentType::None {
            base += (&format!("\r\nContent-Type: {:?}", self.content_type));
        }
        if self.content_length > 0 {
            base += (&format!("\r\nContent-Length: {}", self.content_length));
            base += (&format!("\r\n\r\n{}", self.content));
        } else {
            base += "\r\n\r\n"
        }

        return write!(f, "{}", base)
    }
}