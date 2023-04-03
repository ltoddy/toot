use super::{HttpVersion, StatusCode, CRLF, Headers};

#[derive(Debug)]
pub struct RawResponse {
    status_line: StatusLine,
    headers: Headers,
    body: Option<Vec<u8>>,
}

impl RawResponse {
    pub fn new(status_line: StatusLine, headers: Headers, body: Option<Vec<u8>>) -> Self {
        Self { status_line, headers, body }
    }
}

/// response status line
/// e.g.
/// HTTP/1.1 200 OK
#[derive(Debug)]
pub struct StatusLine {
    version: HttpVersion,
    status: StatusCode,
}

impl StatusLine {
    pub fn new(version: HttpVersion, status: StatusCode) -> Self {
        Self { version, status }
    }

    pub fn as_http_message(&self) -> String {
        let code = self.status.0;
        let phrase = self.status.default_reason_phrase();

        format!("{} {} {}{CRLF}", self.version.as_str(), code, phrase)
    }
}
