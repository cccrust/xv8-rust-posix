use crate::{Body, HeaderMap, StatusCode};
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Response<B = Body> {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: B,
}

impl Response<Body> {
    pub fn new(status: StatusCode) -> Self {
        Response {
            status,
            headers: HeaderMap::new(),
            body: Body::new(),
        }
    }

    pub fn header(mut self, name: &str, value: &[u8]) -> Self {
        let hn = crate::HeaderName::from_bytes(name.as_bytes()).unwrap();
        let hv = crate::HeaderValue::from_bytes(value);
        self.headers.insert(hn, hv);
        self
    }

    pub fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let status_line = format!("HTTP/1.1 {}\r\n", self.status);
        let mut buf = Vec::new();
        buf.extend_from_slice(status_line.as_bytes());

        let con_len = self.body.len();
        buf.extend_from_slice(b"content-length: ");
        buf.extend_from_slice(con_len.to_string().as_bytes());
        buf.extend_from_slice(b"\r\n");

        if let Some(ct) = self.headers.get("content-type") {
            buf.extend_from_slice(b"content-type: ");
            buf.extend_from_slice(ct.as_bytes());
            buf.extend_from_slice(b"\r\n");
        } else if con_len > 0 {
            buf.extend_from_slice(b"content-type: text/plain\r\n");
        }

        for (name, value) in self.headers.iter() {
            let n = name.as_str();
            if n != "content-length" && n != "content-type" {
                buf.extend_from_slice(n.as_bytes());
                buf.extend_from_slice(b": ");
                buf.extend_from_slice(value.as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
        }

        buf.extend_from_slice(b"\r\n");
        buf.extend_from_slice(self.body.as_bytes());
        buf
    }
}
