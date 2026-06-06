use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(u16);

impl StatusCode {
    pub const fn new(code: u16) -> Self {
        StatusCode(code)
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn reason_phrase(&self) -> &str {
        match self.0 {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            206 => "Partial Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            307 => "Temporary Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "Unknown",
        }
    }

    pub const OK: StatusCode = StatusCode(200);
    pub const CREATED: StatusCode = StatusCode(201);
    pub const NO_CONTENT: StatusCode = StatusCode(204);
    pub const MOVED_PERMANENTLY: StatusCode = StatusCode(301);
    pub const NOT_MODIFIED: StatusCode = StatusCode(304);
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    pub const METHOD_NOT_ALLOWED: StatusCode = StatusCode(405);
    pub const REQUEST_TIMEOUT: StatusCode = StatusCode(408);
    pub const PAYLOAD_TOO_LARGE: StatusCode = StatusCode(413);
    pub const URI_TOO_LONG: StatusCode = StatusCode(414);
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode(500);
    pub const NOT_IMPLEMENTED: StatusCode = StatusCode(501);
    pub const SERVICE_UNAVAILABLE: StatusCode = StatusCode(503);
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, self.reason_phrase())
    }
}
