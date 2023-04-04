use std::fmt::{Display, Formatter};
use std::io;
use std::io::Cursor;
use std::io::Write;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{Headers, HttpVersion, StatusCode, CRLF};

pub async fn write_http_response<W>(writer: &mut W, response: RawResponse) -> io::Result<()>
where
    W: AsyncWrite + ?Sized + Unpin,
{
    let message = response.into_vec();
    writer.write_all(&message).await?;
    Ok(())
}

#[derive(Debug)]
pub struct RawResponse {
    status_line: StatusLine,
    headers: Headers,
    body: Option<Vec<u8>>,
}

impl RawResponse {
    pub fn new(status_line: StatusLine, headers: Headers, body: Option<Vec<u8>>) -> Self {
        let mut headers = headers;
        if let Some(ref body) = body {
            headers.set("Content-Length", body.len().to_string())
        };

        Self { status_line, headers, body }
    }

    pub fn into_vec(self) -> Vec<u8> {
        let Self { status_line, headers, body } = self;
        let buffer = Vec::<u8>::with_capacity(512);
        let mut cursor = Cursor::new(buffer);

        let _ = write!(cursor, "{}", status_line.to_http_message());
        let _ = write!(cursor, "{}", headers.to_http_message());
        let _ = write!(cursor, "{CRLF}");
        if let Some(body) = body {
            let _ = Write::write_all(&mut cursor, &body);
        }

        cursor.into_inner()
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

    pub fn to_http_message(&self) -> String {
        format!(
            "{} {} {}{CRLF}",
            self.version.as_str(),
            self.status.0,
            self.status.default_reason_phrase()
        )
    }
}
