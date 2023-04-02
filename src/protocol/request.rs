use std::fmt::{Display, Formatter};
use std::io;
use std::str::FromStr;

use tokio::io::{AsyncRead, AsyncReadExt};

use super::{HttpVersion, Method, ParseRequestError};

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

#[derive(Debug)]
pub struct RequestLine {
    method: Method,
    uri: String,
    version: HttpVersion,
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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    pub async fn test_parse_request_line() {
        let line = "GET /some HTTP/1.1\r\n";
        let line = read_next_line(&mut line.as_bytes()).await.unwrap();
        let request_line = String::from_utf8_lossy(&line)
            .parse::<RequestLine>()
            .unwrap();

        assert_eq!(Method::GET, request_line.method);
        assert_eq!(HttpVersion::Http1_1, request_line.version);
        assert_eq!("/some", request_line.uri);
    }
}
