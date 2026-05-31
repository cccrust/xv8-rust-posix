use alloc::string::{String, ToString};
use crate::io::Read;

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
    pub fn from_fd(fd: usize) -> super::io::Result<Self> {
        let mut stat = xv8_libc::Stat::default();
        let n = xv8_libc::fstat(fd, &mut stat as *mut _);
        if n < 0 { return Err(super::io::ErrorKind::Other.into()); }
        Ok(Metadata {
            mode: stat.mode,
            uid: stat.uid,
            gid: stat.gid,
            size: stat.size,
            mtime: 0,
            nlink: stat.nlink as u32,
            ino: stat.ino,
        })
    }
    pub fn is_dir(&self) -> bool { (self.mode & 0o170000) == 0o40000 }
    pub fn is_file(&self) -> bool { (self.mode & 0o170000) == 0o100000 }
    pub fn permissions(&self) -> Permissions { Permissions { mode: self.mode } }
    pub fn len(&self) -> u64 { self.size }
}

#[derive(Clone, Copy)]
pub struct Permissions { pub mode: u32 }
impl Permissions {
    pub fn mode(&self) -> u32 { self.mode }
    pub fn set_mode(&mut self, mode: u32) { self.mode = mode; }
}

pub struct File { fd: usize }

impl File {
    pub fn open(path: &super::path::Path) -> super::io::Result<Self> {
        let path_str = path.to_str().unwrap_or("");
        let fd = xv8_libc::open(path_str.as_ptr(), xv8_libc::OpenFlag::READ_ONLY);
        if fd < 0 { Err(super::io::ErrorKind::NotFound.into()) } else { Ok(File { fd: fd as usize }) }
    }
    pub fn create(path: &super::path::Path) -> super::io::Result<Self> {
        let path_str = path.to_str().unwrap_or("");
        let flags = xv8_libc::OpenFlag::WRITE_ONLY | xv8_libc::OpenFlag::CREATE | xv8_libc::OpenFlag::TRUNCATE;
        let fd = xv8_libc::open(path_str.as_ptr(), flags);
        if fd < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(File { fd: fd as usize }) }
    }
    pub fn read_raw(&self, buf: &mut [u8]) -> super::io::Result<usize> {
        let n = xv8_libc::read(self.fd, buf.as_mut_ptr(), buf.len());
        if n < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    pub fn write_raw(&self, buf: &[u8]) -> super::io::Result<usize> {
        let n = xv8_libc::write(self.fd, buf.as_ptr(), buf.len());
        if n < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    pub fn metadata(&self) -> super::io::Result<Metadata> { Metadata::from_fd(self.fd) }
    pub fn sync_all(&self) -> super::io::Result<()> { Ok(()) }
    pub fn sync_data(&self) -> super::io::Result<()> { Ok(()) }
    pub fn set_len(&self, _size: u64) -> super::io::Result<()> { Ok(()) }
}

impl super::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> super::io::Result<usize> {
        let n = xv8_libc::read(self.fd, buf.as_mut_ptr(), buf.len());
        if n < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
}

impl super::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> super::io::Result<usize> {
        let n = xv8_libc::write(self.fd, buf.as_ptr(), buf.len());
        if n < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> super::io::Result<()> { Ok(()) }
}

impl Drop for File {
    fn drop(&mut self) {
        xv8_libc::close(self.fd);
    }
}

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
    pub fn open(&self, path: &super::path::Path) -> super::io::Result<File> {
        let path_str = path.to_str().unwrap_or("");
        let mut flags = 0usize;
        if self.read && self.write { flags |= xv8_libc::OpenFlag::READ_WRITE; }
        else if self.write { flags |= xv8_libc::OpenFlag::WRITE_ONLY; }
        else { flags |= xv8_libc::OpenFlag::READ_ONLY; }
        if self.create { flags |= xv8_libc::OpenFlag::CREATE; }
        if self.truncate { flags |= xv8_libc::OpenFlag::TRUNCATE; }
        if self.append { flags |= xv8_libc::OpenFlag::APPEND; }
        let fd = xv8_libc::open(path_str.as_ptr(), flags);
        if fd < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(File { fd: fd as usize }) }
    }
}

impl Default for OpenOptions {
    fn default() -> Self { Self::new() }
}

pub struct ReadDir { _path: String, _offset: usize }
pub struct DirEntry { _name: String, _metadata: Metadata }

impl Iterator for ReadDir {
    type Item = super::io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> { None }
}

pub fn read_to_string(path: &super::path::Path) -> super::io::Result<String> {
    let mut file = File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

pub fn read_dir(path: &super::path::Path) -> super::io::Result<ReadDir> {
    Ok(ReadDir { _path: path.to_str().unwrap_or("").to_string(), _offset: 0 })
}
