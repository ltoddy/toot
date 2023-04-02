use std::fmt::{Display, Formatter};
use std::io;
use std::str::FromStr;

use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Headers, HttpVersion, Method, ParseRequestError};

pub async fn read_http_request<R>(reader: &mut R) -> Result<RawRequest, ParseRequestError>
where
    R: AsyncRead + ?Sized + Unpin,
{
    let line = read_next_line(reader).await?;
    let request_line = String::from_utf8_lossy(&line).parse::<RequestLine>()?;

    let mut headers = Headers::empty();
    loop {
        let line = read_next_line(reader).await?;
        if line.is_empty() {
            break;
        }
        let header = String::from_utf8_lossy(&line).parse()?;
        headers.push(header);
    }

    let body = {
        if let Some(length) = headers.get("Content-Length").and_then(|v| v.parse::<usize>().ok()) {
            let mut body = vec![0; length];
            reader.read_exact(&mut body).await?;
            Some(body)
        } else {
            None
        }
    };

    let request = RawRequest { request_line, headers, body };
    Ok(request)
}

/// Reads until `CRLF` is reached
async fn read_next_line<R>(reader: &mut R) -> io::Result<Vec<u8>>
where
    R: AsyncRead + ?Sized + Unpin,
{
    let mut line = Vec::<u8>::with_capacity(16);
    let mut prev_byte_was_cr = false;

    loop {
        let byte = reader.read_u8().await?;

        if byte == b'\n' && prev_byte_was_cr {
            line.pop();
            return Ok(line);
        }

        prev_byte_was_cr = byte == b'\r';
        line.push(byte);
    }
}

pub struct RawRequest {
    pub request_line: RequestLine,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct RequestLine {
    pub method: Method,
    pub uri: String,
    pub version: HttpVersion,
}

impl Display for RequestLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let method = self.method.as_str();
        let uri = self.uri.as_str();
        let version = self.version.as_str();

        write!(f, "{method} {uri} {version}")
    }
}

impl FromStr for RequestLine {
    type Err = ParseRequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(' ');

        let method = parts.next().and_then(|w| w.parse::<Method>().ok());
        let uri = parts.next().map(|w| w.to_owned());
        let version = parts.next().and_then(|w| w.parse::<HttpVersion>().ok());

        let request_line = RequestLine {
            method: method.ok_or(ParseRequestError::RequestLine(s.to_owned()))?,
            uri: uri.ok_or(ParseRequestError::RequestLine(s.to_owned()))?,
            version: version.ok_or(ParseRequestError::RequestLine(s.to_owned()))?,
        };
        Ok(request_line)
    }
}
