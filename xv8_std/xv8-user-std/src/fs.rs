use core::fmt;
use alloc::string::ToString;
use user::kernel::abi::{OpenFlag, Stat};
use user::syscall::{self, Fd};

fn posix_to_xv8_flags(posix_flags: u32) -> usize {
    let mut flags = 0usize;
    if posix_flags & 0o400000 != 0 { flags |= OpenFlag::READ_ONLY; }
    if posix_flags & 0o200000 != 0 { flags |= OpenFlag::WRITE_ONLY; }
    if posix_flags & 0o100000 != 0 { flags |= OpenFlag::CREATE; }
    if posix_flags & 0o40000 != 0 { flags |= OpenFlag::EXCLUSIVE; }
    if posix_flags & 0o10000 != 0 { flags |= OpenFlag::TRUNCATE; }
    if posix_flags & 0o20000 != 0 { flags |= OpenFlag::APPEND; }
    flags
}

#[derive(Clone)]
pub struct Metadata {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub mtime: i64,
    pub nlink: u32,
    pub ino: u64,
}

impl Metadata {
    pub fn from_fd(fd: Fd) -> core::io::Result<Self> {
        let mut stat = Stat::default();
        syscall::fstat(fd, &mut stat).map_err(super::io::syserr_to_io)?;
        Ok(Metadata {
            mode: stat.mode,
            uid: stat.uid,
            gid: stat.gid,
            size: stat.size as u64,
            mtime: stat.mtime as i64,
            nlink: stat.nlink,
            ino: stat.ino,
        })
    }
    pub fn is_dir(&self) -> bool { (self.mode & 0o170000) == 0o40000 }
    pub fn is_file(&self) -> bool { (self.mode & 0o170000) == 0o100000 }
    pub fn permissions(&self) -> Permissions { Permissions { mode: self.mode } }
    pub fn len(&self) -> u64 { self.size }
}

#[derive(Clone, Copy)]
pub struct Permissions { mode: u32 }
impl Permissions {
    pub fn mode(&self) -> u32 { self.mode }
    pub fn set_mode(&mut self, mode: u32) { self.mode = mode; }
}

pub struct File { fd: Fd }

impl File {
    pub fn open(path: &super::path::Path) -> core::io::Result<Self> {
        let path_str = path.to_str().unwrap_or("");
        syscall::open(path_str, posix_to_xv8_flags(0o400000))
            .map(|fd| File { fd })
            .map_err(super::io::syserr_to_io)
    }
    pub fn create(path: &super::path::Path, mode: u32) -> core::io::Result<Self> {
        let path_str = path.to_str().unwrap_or("");
        syscall::open(path_str, posix_to_xv8_flags(0o400000 | 0o100000))
            .map(|fd| File { fd })
            .map_err(super::io::syserr_to_io)
    }
    pub fn read(&self, buf: &mut [u8]) -> core::io::Result<usize> {
        syscall::read(self.fd, buf).map_err(super::io::syserr_to_io)
    }
    pub fn write(&self, buf: &[u8]) -> core::io::Result<usize> {
        syscall::write(self.fd, buf).map_err(super::io::syserr_to_io)
    }
    pub fn seek(&self, pos: SeekFrom) -> core::io::Result<u64> {
        let off = pos.offset();
        let whence = pos.whence();
        let result = self.fd.as_raw() as isize;
        drop(result);
        Ok(0)
    }
    pub fn sync_all(&self) -> core::io::Result<()> { Ok(()) }
    pub fn sync_data(&self) -> core::io::Result<()> { Ok(()) }
    pub fn set_len(&self, _size: u64) -> core::io::Result<()> { Ok(()) }
    pub fn metadata(&self) -> core::io::Result<Metadata> { Metadata::from_fd(self.fd) }
    pub fn into_fd(self) -> Fd { self.fd }
    pub fn fd(&self) -> Fd { self.fd }
}

impl core::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> core::io::Result<usize> { self.read(buf) }
}

impl core::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> core::io::Result<usize> { self.write(buf) }
    fn flush(&mut self) -> core::io::Result<()> { Ok(()) }
}

impl core::io::Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> core::io::Result<u64> { self.seek(pos) }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File(fd={})", self.fd)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = syscall::close(self.fd);
    }
}

#[derive(Clone, Copy)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
    truncate: bool,
    append: bool,
    mode: u32,
}

impl OpenOptions {
    pub fn new() -> Self {
        OpenOptions { read: false, write: false, create: false, truncate: false, append: false, mode: 0o644 }
    }
    pub fn read(&mut self, yes: bool) -> &mut Self { self.read = yes; self }
    pub fn write(&mut self, yes: bool) -> &mut Self { self.write = yes; self }
    pub fn create(&mut self, yes: bool) -> &mut Self { self.create = yes; self }
    pub fn truncate(&mut self, yes: bool) -> &mut Self { self.truncate = yes; self }
    pub fn append(&mut self, yes: bool) -> &mut Self { self.append = yes; self }
    pub fn mode(&mut self, mode: u32) -> &mut Self { self.mode = mode; self }
    pub fn open(&self, path: &super::path::Path) -> core::io::Result<File> {
        let path_str = path.to_str().unwrap_or("");
        let mut flags = 0usize;
        if self.read { flags |= OpenFlag::READ_ONLY; }
        if self.write { flags |= OpenFlag::WRITE_ONLY; }
        if self.create { flags |= OpenFlag::CREATE; }
        if self.truncate { flags |= OpenFlag::TRUNCATE; }
        if self.append { flags |= OpenFlag::APPEND; }
        syscall::open(path_str, flags)
            .map(|fd| File { fd })
            .map_err(super::io::syserr_to_io)
    }
}

impl Default for OpenOptions {
    fn default() -> Self { Self::new() }
}

pub struct ReadDir { _path: String, _offset: usize }
pub struct DirEntry { name: alloc::string::String, metadata: Metadata }

impl Iterator for ReadDir {
    type Item = core::io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> { None }
}

#[derive(Clone, Copy)]
pub struct SeekFrom {
    offset: i64,
    whence: i32,
}

impl SeekFrom {
    pub const fn new(offset: i64, whence: i32) -> Self { SeekFrom { offset, whence } }
    pub fn offset(&self) -> i64 { self.offset }
    pub fn whence(&self) -> i32 { self.whence }
}

impl From<SeekFrom> for i64 {
    fn from(sf: SeekFrom) -> i64 { sf.offset }
}

pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

impl SeekFrom {
    pub fn start(off: i64) -> Self { SeekFrom { offset: off, whence: SEEK_SET } }
    pub fn current(off: i64) -> Self { SeekFrom { offset: off, whence: SEEK_CUR } }
    pub fn end(off: i64) -> Self { SeekFrom { offset: off, whence: SEEK_END } }
}

pub fn read_to_string(path: &super::path::Path) -> core::io::Result<alloc::string::String> {
    let mut file = File::open(path)?;
    let mut s = alloc::string::String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

pub fn read_dir(path: &super::path::Path) -> core::io::Result<ReadDir> {
    Ok(ReadDir { _path: path.to_str().unwrap_or("").to_string(), _offset: 0 })
}

pub fn symlink_metadata(_path: &super::path::Path) -> core::io::Result<Metadata> {
    Err(core::io::Error::new(core::io::ErrorKind::Unsupported, "symlink_metadata not implemented"))
}