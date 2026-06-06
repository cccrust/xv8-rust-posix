use crate::{Body, HeaderMap, Method, Uri};

#[derive(Debug)]
pub struct Request<B = Body> {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub body: B,
}

impl Request<Body> {
    pub fn new(method: Method, uri: Uri) -> Self {
        Request {
            method,
            uri,
            headers: HeaderMap::new(),
            body: Body::new(),
        }
    }

    pub fn header(&mut self, name: &str, value: &[u8]) -> &mut Self {
        let hn = crate::HeaderName::from_bytes(name.as_bytes()).unwrap();
        let hv = crate::HeaderValue::from_bytes(value);
        self.headers.insert(hn, hv);
        self
    }

    pub fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }
}
