use crate::{Body, HeaderMap, HeaderName, HeaderValue, Method, Request, Uri};

#[derive(Debug)]
pub enum ParseError {
    Incomplete,
    InvalidMethod,
    InvalidUri,
    InvalidVersion,
    InvalidHeader,
    TooLarge,
}

pub fn parse_request(bytes: &[u8]) -> Result<(Request<Body>, usize), ParseError> {
    let mut pos = 0;
    let len = bytes.len();

    if len < 4 {
        return Err(ParseError::Incomplete);
    }

    // --- Method ---
    let method_end = find_token_end(bytes, pos)?;
    let method = Method::from_bytes(&bytes[pos..method_end]).ok_or(ParseError::InvalidMethod)?;
    pos = skip_spaces(bytes, method_end)?;

    // --- URI ---
    let uri_end = find_token_end(bytes, pos)?;
    let uri = Uri::from_bytes(&bytes[pos..uri_end]).ok_or(ParseError::InvalidUri)?;
    pos = skip_spaces(bytes, uri_end)?;

    // --- Version ---
    if pos + 8 > len {
        return Err(ParseError::Incomplete);
    }
    if &bytes[pos..pos + 5] != b"HTTP/" {
        return Err(ParseError::InvalidVersion);
    }
    let ver_end = find_line_end(bytes, pos)?;
    pos = ver_end;

    // --- Headers ---
    let mut headers = HeaderMap::new();
    let body_start;
    loop {
        if pos >= len {
            return Err(ParseError::Incomplete);
        }
        if bytes[pos] == b'\r' {
            if pos + 1 < len && bytes[pos + 1] == b'\n' {
                body_start = pos + 2;
                break;
            }
        }
        if bytes[pos] == b'\n' {
            body_start = pos + 1;
            break;
        }

        let line_end = find_line_end(bytes, pos)?;
        let line = &bytes[pos..line_end];

        let colon = line.iter().position(|&b| b == b':')
            .ok_or(ParseError::InvalidHeader)?;
        let name = HeaderName::from_bytes(&line[..colon])
            .ok_or(ParseError::InvalidHeader)?;
        let mut val_start = colon + 1;
        while val_start < line.len() && line[val_start] == b' ' {
            val_start += 1;
        }
        let value = if val_start < line.len() {
            HeaderValue::from_bytes(&line[val_start..])
        } else {
            HeaderValue::from_bytes(b"")
        };

        headers.insert(name, value);
        pos = line_end;
    }

    // --- Body ---
    let body = if let Some(cl) = headers.get("content-length") {
        let cl_str = core::str::from_utf8(cl.as_bytes()).map_err(|_| ParseError::InvalidHeader)?;
        let cl_val: usize = cl_str.trim().parse().map_err(|_| ParseError::InvalidHeader)?;
        if cl_val > 16384 {
            return Err(ParseError::TooLarge);
        }
        if body_start + cl_val > len {
            return Err(ParseError::Incomplete);
        }
        Body::from_bytes(&bytes[body_start..body_start + cl_val])
    } else {
        Body::new()
    };

    let consumed = body_start + body.len();
    Ok((
        Request {
            method,
            uri,
            headers,
            body,
        },
        consumed,
    ))
}

fn find_token_end(bytes: &[u8], start: usize) -> Result<usize, ParseError> {
    let mut i = start;
    while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b'\r' && bytes[i] != b'\n' {
        i += 1;
    }
    if i == start {
        Err(ParseError::Incomplete)
    } else {
        Ok(i)
    }
}

fn skip_spaces(bytes: &[u8], start: usize) -> Result<usize, ParseError> {
    let mut i = start;
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    if i >= bytes.len() {
        Err(ParseError::Incomplete)
    } else {
        Ok(i)
    }
}

fn find_line_end(bytes: &[u8], start: usize) -> Result<usize, ParseError> {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                return Ok(i + 2);
            }
            return Err(ParseError::InvalidHeader);
        }
        if bytes[i] == b'\n' {
            return Ok(i + 1);
        }
        i += 1;
    }
    Err(ParseError::Incomplete)
}
