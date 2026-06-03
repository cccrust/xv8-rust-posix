pub trait AsRawFd {
    fn as_raw_fd(&self) -> i32;
}

impl AsRawFd for crate::io::Stdin {
    fn as_raw_fd(&self) -> i32 { 0 }
}

impl AsRawFd for crate::io::Stdout {
    fn as_raw_fd(&self) -> i32 { 1 }
}

impl AsRawFd for crate::io::Stderr {
    fn as_raw_fd(&self) -> i32 { 2 }
}
