use std::fmt::{Display, Formatter};
use std::io;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

pub use self::request::{read_http_request, RawRequest, RequestLine};

mod request;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ParseRequestError {
    Io(io::ErrorKind),
    UnknownMethod(String),
    UnknownHttpVersion(String),
    RequestLine(String),
    InvalidHeader(String),
}

impl Display for ParseRequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseRequestError::Io(err) => write!(f, "read failure: {err}"),
            ParseRequestError::UnknownMethod(m) => write!(f, "unknown http method: {m}"),
            ParseRequestError::UnknownHttpVersion(v) => write!(f, "unknown http version: {v}"),
            ParseRequestError::RequestLine(src) => write!(f, "invalid request line: {src}"),
            ParseRequestError::InvalidHeader(src) => {
                write!(f, "invalid characters in header content: {src}")
            }
        }
    }
}

impl From<io::Error> for ParseRequestError {
    fn from(value: io::Error) -> Self {
        ParseRequestError::Io(value.kind())
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
}

#[derive(Debug)]
pub struct Headers(Vec<Header>);

impl Headers {
    pub fn empty() -> Self {
        let inner = Vec::with_capacity(8);
        Self(inner)
    }

    // TODO
    pub fn get(&self, field: &str) -> Option<&str> {
        self.iter().find(|h| h.field.eq_ignore_ascii_case(field)).map(|h| h.value.as_ref())
    }
}

impl Deref for Headers {
    type Target = Vec<Header>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for Header { field, value } in self.iter() {
            writeln!(f, "{}: {}", field.as_str(), value.as_str())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Header {
    field: String,
    value: String,
}

impl Header {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl FromStr for Header {
    type Err = ParseRequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(": ");

        let field = parts
            .next()
            .map(|w| w.to_owned())
            .ok_or(ParseRequestError::InvalidHeader(s.to_owned()))?;
        let value = parts
            .next()
            .map(|w| w.trim())
            .map(|w| w.to_owned())
            .ok_or(ParseRequestError::InvalidHeader(s.to_owned()))?;

        let header = Header { field, value };
        Ok(header)
    }
}
