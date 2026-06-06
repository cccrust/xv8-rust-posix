use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct HeaderName(String);

impl HeaderName {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let s = core::str::from_utf8(bytes).ok()?;
        Some(HeaderName(s.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct HeaderValue(Vec<u8>);

impl HeaderValue {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        HeaderValue(bytes.to_vec())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone)]
pub struct HeaderMap {
    headers: Vec<(HeaderName, HeaderValue)>,
}

impl HeaderMap {
    pub fn new() -> Self {
        HeaderMap {
            headers: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: HeaderName, value: HeaderValue) {
        self.headers.push((name, value));
    }

    pub fn get(&self, name: &str) -> Option<&HeaderValue> {
        let lower = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(n, _)| n.as_str() == lower)
            .map(|(_, v)| v)
    }

    pub fn iter(&self) -> core::slice::Iter<'_, (HeaderName, HeaderValue)> {
        self.headers.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}
