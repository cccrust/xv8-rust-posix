use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Uri {
    pub path: String,
    pub query: String,
}

impl Uri {
    pub fn new(path: &str, query: &str) -> Self {
        Uri {
            path: path.into(),
            query: query.into(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let s = core::str::from_utf8(bytes).ok()?;
        Some(s.into())
    }
}

impl From<&str> for Uri {
    fn from(s: &str) -> Self {
        if let Some(idx) = s.find('?') {
            Uri {
                path: s[..idx].into(),
                query: s[idx + 1..].into(),
            }
        } else {
            Uri {
                path: s.into(),
                query: String::new(),
            }
        }
    }
}
