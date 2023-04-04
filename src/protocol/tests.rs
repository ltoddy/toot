use super::*;

#[test]
pub fn test_parse_http_method() {
    let source = "GET";
    let method = source.parse::<Method>().unwrap();
    assert_eq!(Method::GET, method);

    let source = "another";
    let method = source.parse::<Method>().expect_err("");
    assert_eq!(ParseRequestError::UnknownMethod("another".to_string()), method);
}

#[test]
pub fn test_parse_http_version() {
    let source = "HTTP/1.1";
    let version = source.parse::<HttpVersion>().unwrap();

    assert_eq!(HttpVersion::Http1_1, version);
}

#[test]
pub fn test_parse_http_header() {
    let source = "Content-Type: application/json";
    let header = source.parse::<Header>().unwrap();

    assert_eq!("Content-Type", header.field());
    assert_eq!("application/json", header.value());
}

#[tokio::test]
pub async fn test_parse_request_line() {
    let source = "GET /some HTTP/1.1";
    let request_line = source.parse::<RequestLine>().unwrap();

    assert_eq!(Method::GET, request_line.method);
    assert_eq!(HttpVersion::Http1_1, request_line.version);
    assert_eq!("/some", request_line.uri);
}

#[tokio::test]
pub async fn test_read_http_request() {
    let mut source: &[u8] = b"GET /foo/bar HTTP/1.1\r\nContent-Type: application/json\r\n\r\n";

    let request = read_http_request(&mut source).await.unwrap();

    assert_eq!(Method::GET, request.request_line.method);
    assert_eq!(HttpVersion::Http1_1, request.request_line.version);
    assert_eq!("/foo/bar", request.request_line.uri);
    assert_eq!(1, request.headers.len());
    assert_eq!(Some("application/json"), request.headers.get("Content-Type"));
    assert_eq!(None, request.body);
}

#[test]
pub fn test_status_line_to_http_message() {
    let status_line = StatusLine::new(HttpVersion::Http1_1, StatusCode::OK);

    let bytes = "HTTP/1.1 200 OK\r\n";

    assert_eq!(bytes, status_line.to_http_message());
}

#[test]
pub fn test_header_to_http_message() {
    let header = Header::new("hello", "world");

    let actual = header.to_http_message();

    let expected = "hello: world\r\n";
    assert_eq!(expected, actual);
}
