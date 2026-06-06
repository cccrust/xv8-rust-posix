use alloc::vec::Vec;
use xv8_http::{Body, Response, StatusCode};

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::new(StatusCode::OK)
            .header("content-type", b"text/plain")
            .body(self.into())
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        Response::new(StatusCode::OK)
            .header("content-type", b"application/octet-stream")
            .body(Body::from_bytes(self))
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        Response::new(StatusCode::OK)
            .header("content-type", b"application/octet-stream")
            .body(Body::from_bytes(&self))
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::new(self)
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, T) {
    fn into_response(self) -> Response {
        let mut resp = self.1.into_response();
        resp.status = self.0;
        resp
    }
}

pub fn into_response<T: IntoResponse>(val: T) -> Response {
    val.into_response()
}
