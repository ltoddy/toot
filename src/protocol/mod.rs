use std::fmt::{Display, Formatter};
use std::io;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

pub use self::request::{read_http_request, RawRequest, RequestLine};

mod request;
mod response;
#[cfg(test)]
mod tests;

pub const CRLF: &str = "\r\n";

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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct StatusCode(u16);

impl Deref for StatusCode {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StatusCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u16> for StatusCode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl StatusCode {
    /// 100 Continue
    pub const CONTINUE: StatusCode = StatusCode(100);
    /// 101 Switching Protocols
    pub const SWITCHING_PROTOCOLS: StatusCode = StatusCode(101);
    /// 102 Processing
    pub const PROCESSING: StatusCode = StatusCode(102);
    /// 103 Early Hints
    pub const EARLY_HINTS: StatusCode = StatusCode(103);
    /// 200 OK
    pub const OK: StatusCode = StatusCode(200);
    /// 201 Created
    pub const CREATED: StatusCode = StatusCode(201);
    /// 202 Accepted
    pub const ACCEPTED: StatusCode = StatusCode(202);
    /// 203 Non-Authoritative Information
    pub const NON_AUTHORITATIVE_INFORMATION: StatusCode = StatusCode(203);
    /// 204 No Content
    pub const NO_CONTENT: StatusCode = StatusCode(204);
    /// 205 Reset Content
    pub const RESET_CONTENT: StatusCode = StatusCode(205);
    /// 206 Partial Content
    pub const PARTIAL_CONTENT: StatusCode = StatusCode(206);
    /// 207 Multi-Status
    pub const MULTI_STATUS: StatusCode = StatusCode(207);
    /// 208 Already Reported
    pub const ALREADY_REPORTED: StatusCode = StatusCode(208);
    /// 226 IM Used
    pub const IM_USED: StatusCode = StatusCode(226);
    /// 300 Multiple Choices
    pub const MULTIPLE_CHOICES: StatusCode = StatusCode(300);
    /// 301 Moved Permanently
    pub const MOVED_PERMANENTLY: StatusCode = StatusCode(301);
    /// 302 Found
    pub const FOUND: StatusCode = StatusCode(302);
    /// 303 See Other
    pub const SEE_OTHER: StatusCode = StatusCode(303);
    /// 304 Not Modified
    pub const NOT_MODIFIED: StatusCode = StatusCode(304);
    /// 305 Use Proxy
    pub const USE_PROXY: StatusCode = StatusCode(305);
    /// 307 Temporary Redirect
    pub const TEMPORARY_REDIRECT: StatusCode = StatusCode(307);
    /// 308 Permanent Redirect
    pub const PERMANENT_REDIRECT: StatusCode = StatusCode(308);
    /// 400 Bad Request
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    /// 401 Unauthorized
    pub const UNAUTHORIZED: StatusCode = StatusCode(401);
    /// 402 Payment Required
    pub const PAYMENT_REQUIRED: StatusCode = StatusCode(402);
    /// 403 Forbidden
    pub const FORBIDDEN: StatusCode = StatusCode(403);
    /// 404 Not Found
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    /// 405 Method Not Allowed
    pub const METHOD_NOT_ALLOWED: StatusCode = StatusCode(405);
    /// 406 Not Acceptable
    pub const NOT_ACCEPTABLE: StatusCode = StatusCode(406);
    /// 407 Proxy Authentication Required
    pub const PROXY_AUTHENTICATION_REQUIRED: StatusCode = StatusCode(407);
    /// 408 Request Timeout
    pub const REQUEST_TIMEOUT: StatusCode = StatusCode(408);
    /// 409 Conflict
    pub const CONFLICT: StatusCode = StatusCode(409);
    /// 410 Gone
    pub const GONE: StatusCode = StatusCode(410);
    /// 411 Length Required
    pub const LENGTH_REQUIRED: StatusCode = StatusCode(411);
    /// 412 Precondition Failed
    pub const PRECONDITION_FAILED: StatusCode = StatusCode(412);
    /// 413 Payload Too Large
    pub const PAYLOAD_TOO_LARGE: StatusCode = StatusCode(413);
    /// 414 URI Too Long
    pub const URI_TOO_LONG: StatusCode = StatusCode(414);
    /// 415 Unsupported Media Type
    pub const UNSUPPORTED_MEDIA_TYPE: StatusCode = StatusCode(415);
    /// 416 Range Not Satisfiable
    pub const RANGE_NOT_SATISFIABLE: StatusCode = StatusCode(416);
    /// 417 Expectation Failed
    pub const EXPECTATION_FAILED: StatusCode = StatusCode(417);
    /// 421 Misdirected Request
    pub const MISDIRECTED_REQUEST: StatusCode = StatusCode(421);
    /// 422 Unprocessable Entity
    pub const UNPROCESSABLE_ENTITY: StatusCode = StatusCode(422);
    /// 423 Locked
    pub const Locked: StatusCode = StatusCode(423);
    /// 424 Failed Dependency
    pub const FAILED_DEPENDENCY: StatusCode = StatusCode(424);
    /// 426 Upgrade Required
    pub const UPGRADE_REQUIRED: StatusCode = StatusCode(426);
    /// 428 Precondition Required
    pub const PRECONDITION_REQUIRED: StatusCode = StatusCode(428);
    /// 429 Too Many Requests
    pub const TOO_MANY_REQUESTS: StatusCode = StatusCode(429);
    /// 431 Request Header Fields Too Large
    pub const REQUEST_HEADER_FIELDS_TOO_LARGE: StatusCode = StatusCode(431);
    /// 451 Unavailable For Legal Reasons
    pub const UNAVAILABLE_FOR_LEGAL_REASONS: StatusCode = StatusCode(451);
    /// 500 Internal Server Error
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode(500);
    /// 501 Not Implemented
    pub const NOT_IMPLEMENTED: StatusCode = StatusCode(501);
    /// 502 Bad Gateway
    pub const BAD_GATEWAY: StatusCode = StatusCode(502);
    /// 503 Service Unavailable
    pub const ServiceUnavailable: StatusCode = StatusCode(503);
    /// 504 Gateway Timeout
    pub const GATEWAY_TIMEOUT: StatusCode = StatusCode(504);
    /// 505 HTTP Version Not Supported
    pub const HTTP_VERSION_NOT_SUPPORTED: StatusCode = StatusCode(505);
    /// 506 Variant Also Negotiates
    pub const VARIANT_ALSO_NEGOTIATES: StatusCode = StatusCode(506);
    /// 507 Insufficient Storage
    pub const INSUFFICIENT_STORAGE: StatusCode = StatusCode(507);
    /// 508 Loop Detected
    pub const LOOP_DETECTED: StatusCode = StatusCode(508);
    /// 510 Not Extended
    pub const NOT_EXTENDED: StatusCode = StatusCode(510);
    /// 511 Network Authentication Required
    pub const NETWORK_AUTHENTICATION_REQUIRED: StatusCode = StatusCode(511);

    pub fn default_reason_phrase(&self) -> &'static str {
        match *self {
            StatusCode::CONTINUE => "Continue",
            StatusCode::SWITCHING_PROTOCOLS => "Switching Protocols",
            StatusCode::PROCESSING => "Processing",
            StatusCode::EARLY_HINTS => "Early Hints",
            StatusCode::OK => "OK",
            StatusCode::CREATED => "Created",
            StatusCode::ACCEPTED => "Accepted",
            StatusCode::NON_AUTHORITATIVE_INFORMATION => "Non-Authoritative Information",
            StatusCode::NO_CONTENT => "No Content",
            StatusCode::RESET_CONTENT => "Reset Content",
            StatusCode::PARTIAL_CONTENT => "Partial Content",
            StatusCode::MULTI_STATUS => "Multi-Status",
            StatusCode::ALREADY_REPORTED => "Already Reported",
            StatusCode::IM_USED => "IM Used",
            StatusCode::MULTIPLE_CHOICES => "Multiple Choices",
            StatusCode::MOVED_PERMANENTLY => "Moved Permanently",
            StatusCode::FOUND => "Found",
            StatusCode::SEE_OTHER => "See Other",
            StatusCode::NOT_MODIFIED => "Not Modified",
            StatusCode::USE_PROXY => "Use Proxy",
            StatusCode::TEMPORARY_REDIRECT => "Temporary Redirect",
            StatusCode::PERMANENT_REDIRECT => "Permanent Redirect",
            StatusCode::BAD_REQUEST => "Bad Request",
            StatusCode::UNAUTHORIZED => "Unauthorized",
            StatusCode::PAYMENT_REQUIRED => "Payment Required",
            StatusCode::FORBIDDEN => "Forbidden",
            StatusCode::NOT_FOUND => "Not Found",
            StatusCode::METHOD_NOT_ALLOWED => "Method Not Allowed",
            StatusCode::NOT_ACCEPTABLE => "Not Acceptable",
            StatusCode::PROXY_AUTHENTICATION_REQUIRED => "Proxy Authentication Required",
            StatusCode::REQUEST_TIMEOUT => "Request Timeout",
            StatusCode::CONFLICT => "Conflict",
            StatusCode::GONE => "Gone",
            StatusCode::LENGTH_REQUIRED => "Length Required",
            StatusCode::PRECONDITION_FAILED => "Precondition Failed",
            StatusCode::PAYLOAD_TOO_LARGE => "Payload Too Large",
            StatusCode::URI_TOO_LONG => "URI Too Long",
            StatusCode::UNSUPPORTED_MEDIA_TYPE => "Unsupported Media Type",
            StatusCode::RANGE_NOT_SATISFIABLE => "Range Not Satisfiable",
            StatusCode::EXPECTATION_FAILED => "Expectation Failed",
            StatusCode::MISDIRECTED_REQUEST => "Misdirected Request",
            StatusCode::UNPROCESSABLE_ENTITY => "Unprocessable Entity",
            StatusCode::Locked => "Locked",
            StatusCode::FAILED_DEPENDENCY => "Failed Dependency",
            StatusCode::UPGRADE_REQUIRED => "Upgrade Required",
            StatusCode::PRECONDITION_REQUIRED => "Precondition Required",
            StatusCode::TOO_MANY_REQUESTS => "Too Many Requests",
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE => "Request Header Fields Too Large",
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => "Unavailable For Legal Reasons",
            StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error",
            StatusCode::NOT_IMPLEMENTED => "Not Implemented",
            StatusCode::BAD_GATEWAY => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::GATEWAY_TIMEOUT => "Gateway Timeout",
            StatusCode::HTTP_VERSION_NOT_SUPPORTED => "HTTP Version Not Supported",
            StatusCode::VARIANT_ALSO_NEGOTIATES => "Variant Also Negotiates",
            StatusCode::INSUFFICIENT_STORAGE => "Insufficient Storage",
            StatusCode::LOOP_DETECTED => "Loop Detected",
            StatusCode::NOT_EXTENDED => "Not Extended",
            StatusCode::NETWORK_AUTHENTICATION_REQUIRED => "Network Authentication Required",
            _ => "Unknown",
        }
    }
}
