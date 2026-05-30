/// POSIX file mode string (like `ls -l`): `-rwxr-xr-x`
pub fn mode_string(mode: u32) -> [u8; 10] {
    let mut s = [b'-'; 10];
    // File type
    s[0] = match mode & 0o170000 {
        0o100000 => b'-', // regular
        0o040000 => b'd', // directory
        0o120000 => b'l', // symlink
        0o020000 => b'c', // character device
        0o060000 => b'b', // block device
        0o010000 => b'p', // fifo
        0o140000 => b's', // socket
        _ => b'?',
    };
    // Owner
    if mode & 0o400 != 0 { s[1] = b'r'; }
    if mode & 0o200 != 0 { s[2] = b'w'; }
    s[3] = if mode & 0o4000 != 0 { b'S' } else { b'-' };
    if mode & 0o100 != 0 { s[3] = b'x'; }
    if mode & 0o4000 != 0 && mode & 0o100 != 0 { s[3] = b's'; }
    // Group
    if mode & 0o040 != 0 { s[4] = b'r'; }
    if mode & 0o020 != 0 { s[5] = b'w'; }
    s[6] = if mode & 0o2000 != 0 { b'S' } else { b'-' };
    if mode & 0o010 != 0 { s[6] = b'x'; }
    if mode & 0o2000 != 0 && mode & 0o010 != 0 { s[6] = b's'; }
    // Other
    if mode & 0o004 != 0 { s[7] = b'r'; }
    if mode & 0o002 != 0 { s[8] = b'w'; }
    s[9] = if mode & 0o1000 != 0 { b'T' } else { b'-' };
    if mode & 0o001 != 0 { s[9] = b'x'; }
    if mode & 0o1000 != 0 && mode & 0o001 != 0 { s[9] = b't'; }
    s
}

/// Format a number as a string (no allocation).
/// Returns the slice into the provided buffer.
pub fn format_number<'a>(n: u64, buf: &'a mut [u8; 20]) -> &'a str {
    let mut i = 20;
    let mut remaining = n;
    loop {
        i -= 1;
        buf[i] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }
    core::str::from_utf8(&buf[i..]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_string_regular() {
        let s = mode_string(0o100755); // S_IFREG | 0755
        assert_eq!(&s, b"-rwxr-xr-x");
    }

    #[test]
    fn test_mode_string_directory() {
        let s = mode_string(0o40755);
        assert_eq!(&s, b"drwxr-xr-x");
    }

    #[test]
    fn test_mode_string_setuid() {
        let s = mode_string(0o104755); // S_IFREG | 04755
        assert_eq!(&s, b"-rwsr-xr-x");
    }

    #[test]
    fn test_mode_string_sticky() {
        let s = mode_string(0o41777);
        assert_eq!(&s, b"drwxrwxrwt");
    }

    #[test]
    fn test_format_number() {
        let mut buf = [0u8; 20];
        let s = format_number(0, &mut buf);
        assert_eq!(s, "0");
        let s = format_number(12345, &mut buf);
        assert_eq!(s, "12345");
        let s = format_number(999999, &mut buf);
        assert_eq!(s, "999999");
    }
}
