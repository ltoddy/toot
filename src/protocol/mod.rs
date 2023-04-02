use std::fmt::{Display, Formatter};
use std::io;
use std::str::FromStr;

mod request;

#[derive(Debug)]
pub enum ParseRequestError {
    Io(io::Error),
    UnknownMethod(String),
    UnknownHttpVersion(String),
    RequestLine(String),
}

impl Display for ParseRequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseRequestError::Io(err) => write!(f, "read failure: {err}"),
            ParseRequestError::UnknownMethod(m) => write!(f, "unknown http method: {m}"),
            ParseRequestError::UnknownHttpVersion(v) => write!(f, "unknown http version: {v}"),
            ParseRequestError::RequestLine(src) => write!(f, "invalid request line: {src}"),
        }
    }
}

impl std::error::Error for ParseRequestError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            ParseRequestError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ParseRequestError {
    fn from(value: io::Error) -> Self {
        ParseRequestError::Io(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialOrd, Eq, PartialEq)]
pub enum Method {
    /// HTTP GET
    GET,
    /// HTTP HEAD
    HEAD,
    /// HTTP POST
    POST,
    /// HTTP PUT
    PUT,
    /// HTTP DELETE
    DELETE,
    /// HTTP PATCH
    PATCH,
    /// HTTP OPTIONS
    OPTIONS,
    /// HTTP TRACE
    TRACE,
}

impl FromStr for Method {
    type Err = ParseRequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            _ => Err(ParseRequestError::UnknownMethod(s.to_owned())),
        }
    }
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialOrd, Eq, PartialEq)]
pub enum HttpVersion {
    Http0_9,
    Http1_0,
    Http1_1,
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for HttpVersion {
    type Err = ParseRequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/0.9" => Ok(HttpVersion::Http0_9),
            "HTTP/1.0" => Ok(HttpVersion::Http1_0),
            "HTTP/1.1" => Ok(HttpVersion::Http1_1),
            _ => Err(ParseRequestError::UnknownHttpVersion(s.to_owned())),
        }
    }
}

impl HttpVersion {
    pub fn as_str(&self) -> &str {
        match self {
            HttpVersion::Http0_9 => "HTTP/0.9",
            HttpVersion::Http1_0 => "HTTP/1.0",
            HttpVersion::Http1_1 => "HTTP/1.1",
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            HttpVersion::Http0_9 => b"HTTP/0.9",
            HttpVersion::Http1_0 => b"HTTP/1.0",
            HttpVersion::Http1_1 => b"HTTP/1.1",
        }
    }
}
