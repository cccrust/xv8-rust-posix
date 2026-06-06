use alloc::vec::Vec;

#[derive(Debug, Clone, Default)]
pub struct Body(Vec<u8>);

impl Body {
    pub fn new() -> Self {
        Body(Vec::new())
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Body(bytes.to_vec())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        Body(v)
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        Body(s.as_bytes().to_vec())
    }
}
