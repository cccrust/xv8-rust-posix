use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::mem::size_of;
use crate::io::Read;
use crate::ffi::CString;
use crate::os::unix::io::{AsFd, BorrowedFd};

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
            mode: stat.mode as u32,
            uid: stat.uid as u32,
            gid: stat.gid as u32,
            size: stat.size,
            mtime: 0,
            nlink: stat.nlink as u32,
            ino: stat.ino as u64,
        })
    }
    pub fn is_dir(&self) -> bool { (self.mode & 0o170000) == 0o40000 }
    pub fn is_file(&self) -> bool { (self.mode & 0o170000) == 0o100000 }
    pub fn permissions(&self) -> Permissions { Permissions { mode: self.mode } }
    pub fn len(&self) -> u64 { self.size }
    pub fn blocks(&self) -> u64 { (self.size + 511) / 512 }
    pub fn accessed(&self) -> super::io::Result<super::time::SystemTime> { Ok(super::time::UNIX_EPOCH) }
    pub fn modified(&self) -> super::io::Result<super::time::SystemTime> { Ok(super::time::UNIX_EPOCH) }
    pub fn created(&self) -> super::io::Result<super::time::SystemTime> { Ok(super::time::UNIX_EPOCH) }
    pub fn uid(&self) -> u32 { self.uid }
    pub fn gid(&self) -> u32 { self.gid }
    pub fn is_symlink(&self) -> bool { (self.mode & 0o170000) == 0o120000 }
    pub fn file_type(&self) -> FileType {
        match self.mode & 0o170000 {
            0o100000 => FileType::RegularFile,
            0o040000 => FileType::Directory,
            0o020000 => FileType::CharDevice,
            0o060000 => FileType::BlockDevice,
            0o010000 => FileType::Fifo,
            0o140000 => FileType::Socket,
            0o120000 => FileType::Symlink,
            _ => FileType::Other,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Permissions { pub mode: u32 }
impl Permissions {
    pub fn mode(&self) -> u32 { self.mode }
    pub fn set_mode(&mut self, mode: u32) { self.mode = mode; }
    pub fn from_mode(mode: u32) -> Self { Permissions { mode } }
    pub fn readonly(&self) -> bool { self.mode & 0o222 == 0 }
}

pub struct File { fd: usize }

impl File {
    pub fn open<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<Self> {
        let path_str = path.as_ref().to_str().unwrap_or("");
        let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
        let fd = xv8_libc::open(c_path.as_ptr() as *const u8, xv8_libc::OpenFlag::READ_ONLY);
        if fd < 0 { Err(super::io::ErrorKind::NotFound.into()) } else { Ok(File { fd: fd as usize }) }
    }
    pub fn create<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<Self> {
        let path_str = path.as_ref().to_str().unwrap_or("");
        let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
        let flags = xv8_libc::OpenFlag::WRITE_ONLY | xv8_libc::OpenFlag::CREATE | xv8_libc::OpenFlag::TRUNCATE;
        let fd = xv8_libc::open(c_path.as_ptr() as *const u8, flags);
        if fd < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(File { fd: fd as usize }) }
    }
    pub fn options() -> OpenOptions { OpenOptions::new() }
    pub(crate) fn from_raw_fd(fd: usize) -> Self { File { fd } }
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
    pub fn as_raw_fd(&self) -> usize { self.fd }
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

impl super::io::Seek for File {
    fn seek(&mut self, pos: super::io::SeekFrom) -> super::io::Result<u64> {
        let (whence, offset) = match pos {
            super::io::SeekFrom::Start(n) => (0, n as isize),
            super::io::SeekFrom::Current(n) => (1, n as isize),
            super::io::SeekFrom::End(n) => (2, n as isize),
        };
        let n = xv8_libc::lseek(self.fd, offset, whence);
        if n < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(n as u64) }
    }
}

impl AsFd for File {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd() as i32) }
    }
}

pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
    create_new: bool,
    truncate: bool,
    append: bool,
    mode: u32,
}

impl OpenOptions {
    pub fn new() -> Self {
        OpenOptions { read: false, write: false, create: false, create_new: false, truncate: false, append: false, mode: 0o644 }
    }
    pub fn read(&mut self, yes: bool) -> &mut Self { self.read = yes; self }
    pub fn write(&mut self, yes: bool) -> &mut Self { self.write = yes; self }
    pub fn create(&mut self, yes: bool) -> &mut Self { self.create = yes; self }
    pub fn create_new(&mut self, yes: bool) -> &mut Self { self.create_new = yes; self }
    pub fn truncate(&mut self, yes: bool) -> &mut Self { self.truncate = yes; self }
    pub fn append(&mut self, yes: bool) -> &mut Self { self.append = yes; self }
    pub fn mode(&mut self, mode: u32) -> &mut Self { self.mode = mode; self }
    pub fn open<P: AsRef<super::path::Path>>(&self, path: P) -> super::io::Result<File> {
        let path_str = path.as_ref().to_str().unwrap_or("");
        let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
        let mut flags = 0usize;
        if self.read && self.write { flags |= xv8_libc::OpenFlag::READ_WRITE; }
        else if self.write { flags |= xv8_libc::OpenFlag::WRITE_ONLY; }
        else { flags |= xv8_libc::OpenFlag::READ_ONLY; }
        if self.create && self.create_new { return Err(super::io::ErrorKind::AlreadyExists.into()); }
        if self.create || self.create_new { flags |= xv8_libc::OpenFlag::CREATE; }
        if self.truncate { flags |= xv8_libc::OpenFlag::TRUNCATE; }
        if self.append { flags |= xv8_libc::OpenFlag::APPEND; }
        let fd = xv8_libc::open(c_path.as_ptr() as *const u8, flags);
        if fd < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(File { fd: fd as usize }) }
    }
}

impl Default for OpenOptions {
    fn default() -> Self { Self::new() }
}

pub fn write<P: AsRef<super::path::Path>, C: AsRef<[u8]>>(path: P, contents: C) -> super::io::Result<()> {
    use super::io::Write;
    let mut f = File::create(path)?;
    f.write_all(contents.as_ref())
}

pub fn create_dir<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<()> {
    let path_str = path.as_ref().to_str().unwrap_or("");
    let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let ret = xv8_libc::mkdir(c_path.as_ptr() as *const u8, 0o755);
    if ret < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(()) }
}

pub fn remove_file<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<()> {
    let path_str = path.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let ret = xv8_libc::unlink(c_path.as_ptr() as *const u8);
    if ret < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(()) }
}

pub fn remove_dir<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<()> {
    let path_str = path.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let ret = xv8_libc::unlink(c_path.as_ptr() as *const u8);
    if ret < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(()) }
}

pub fn rename<P: AsRef<super::path::Path>, Q: AsRef<super::path::Path>>(from: P, to: Q) -> super::io::Result<()> {
    let from_str = from.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let to_str = to.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let c_from = CString::new(from_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let c_to = CString::new(to_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let ret = xv8_libc::rename(c_from.as_ptr() as *const u8, c_to.as_ptr() as *const u8);
    if ret < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(()) }
}

pub fn copy<P: AsRef<super::path::Path>, Q: AsRef<super::path::Path>>(from: P, to: Q) -> super::io::Result<u64> {
    use super::io::{Read, Write};
    let mut src = File::open(from)?;
    let mut dst = File::create(to)?;
    let mut buf = [0u8; 4096];
    let mut total = 0u64;
    loop {
        let n = src.read(&mut buf)?;
        if n == 0 { break; }
        dst.write_all(&buf[..n])?;
        total += n as u64;
    }
    Ok(total)
}

pub fn hard_link<P: AsRef<super::path::Path>, Q: AsRef<super::path::Path>>(from: P, to: Q) -> super::io::Result<()> {
    let from_str = from.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let to_str = to.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let c_from = CString::new(from_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let c_to = CString::new(to_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let ret = xv8_libc::link(c_from.as_ptr() as *const u8, c_to.as_ptr() as *const u8);
    if ret < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(()) }
}

pub fn metadata<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<Metadata> {
    path.as_ref().metadata()
}

pub fn read_link<P: AsRef<super::path::Path>>(_path: P) -> super::io::Result<super::path::PathBuf> {
    let path = _path.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let c_path = CString::new(path).map_err(|_| super::io::ErrorKind::InvalidInput)?;

    let mut size = 256usize;
    loop {
        let mut buffer = alloc::vec::Vec::with_capacity(size);
        buffer.resize(size, 0);
        let n = xv8_libc::readlink(c_path.as_ptr() as *const u8, buffer.as_mut_ptr(), buffer.len());
        if n < 0 {
            return Err(super::io::ErrorKind::Other.into());
        }
        let n = n as usize;
        if n < buffer.len() {
            buffer.truncate(n);
            return Ok(super::path::PathBuf::from(buffer));
        }
        size = size.saturating_mul(2).max(256);
    }
}

pub fn symlink_metadata<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<Metadata> {
    metadata(path)
}

pub fn set_permissions<P: AsRef<super::path::Path>>(path: P, perm: Permissions) -> super::io::Result<()> {
    let path_str = path.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let ret = xv8_libc::chmod(c_path.as_ptr() as *const u8, perm.mode as usize);
    if ret < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(()) }
}

pub fn create_dir_all<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<()> {
    let path_str = path.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    if path_str.is_empty() || path_str == "/" { return Ok(()); }
    let mut accum = String::new();
    for component in path_str.split('/') {
        if component.is_empty() { continue; }
        accum.push('/');
        accum.push_str(component);
        let c_path = CString::new(accum.as_str()).map_err(|_| super::io::ErrorKind::InvalidInput)?;
        let ret = xv8_libc::mkdir(c_path.as_ptr() as *const u8, 0o755);
        if ret < 0 {
            let err = (-ret) as u16;
            if err == 17 { continue; } // AlreadyExists (EEXIST)
            return Err(super::io::ErrorKind::Other.into());
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub enum FileType {
    RegularFile,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
    Fifo,
    Socket,
    Other,
}

impl FileType {
    pub fn is_dir(&self) -> bool { matches!(self, FileType::Directory) }
    pub fn is_file(&self) -> bool { matches!(self, FileType::RegularFile) }
    pub fn is_symlink(&self) -> bool { matches!(self, FileType::Symlink) }
    pub fn is_block_device(&self) -> bool { matches!(self, FileType::BlockDevice) }
    pub fn is_char_device(&self) -> bool { matches!(self, FileType::CharDevice) }
    pub fn is_fifo(&self) -> bool { matches!(self, FileType::Fifo) }
    pub fn is_socket(&self) -> bool { matches!(self, FileType::Socket) }
}

pub struct ReadDir { file: File, _path: String, _offset: usize }
pub struct DirEntry { _path: String, _name: String, _metadata: Option<Metadata> }

#[repr(C)]
#[derive(Clone, Copy)]
struct Directory {
    inum: u16,
    name: [u8; 14],
}

impl DirEntry {
    pub fn path(&self) -> super::path::PathBuf {
        super::path::PathBuf::from(self._path.as_bytes())
    }
    pub fn file_name(&self) -> &super::ffi::OsStr {
        super::ffi::OsStr::from_str(&self._name)
    }
    pub fn metadata(&self) -> super::io::Result<Metadata> {
        self._metadata.clone().ok_or(super::io::ErrorKind::Other.into())
    }
    pub fn file_type(&self) -> super::io::Result<super::fs::FileType> {
        Ok(super::fs::FileType::RegularFile)
    }
}

impl Iterator for ReadDir {
    type Item = super::io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0u8; size_of::<Directory>()];
        loop {
            let n = match self.file.read_raw(&mut buf) {
                Ok(n) => n,
                Err(e) => return Some(Err(e)),
            };

            if n == 0 {
                return None;
            }

            if n != buf.len() {
                return Some(Err(super::io::ErrorKind::Other.into()));
            }

            let dir = unsafe { &*(buf.as_ptr() as *const Directory) };
            if dir.inum == 0 {
                continue;
            }

            let name_len = dir.name.iter().position(|&c| c == 0).unwrap_or(dir.name.len());
            let name = match core::str::from_utf8(&dir.name[..name_len]) {
                Ok(s) if !s.is_empty() => s.to_string(),
                _ => continue,
            };

            let path = if self._path == "/" {
                format!("/{}", name)
            } else if self._path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", self._path, name)
            };

            let metadata = File::open(&path).ok().and_then(|file| file.metadata().ok());
            return Some(Ok(DirEntry { _path: path, _name: name, _metadata: metadata }));
        }
    }
}

pub fn read<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<alloc::vec::Vec<u8>> {
    let file = File::open(path)?;
    let size = file.metadata()?.len() as usize;
    let mut buf = alloc::vec::Vec::with_capacity(size);
    buf.resize(size, 0);
    file.read_raw(&mut buf)?;
    Ok(buf)
}

pub fn read_to_string<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<String> {
    let file = File::open(path)?;
    let mut s = String::new();
    let mut reader = file;
    reader.read_to_string(&mut s)?;
    Ok(s)
}

pub fn canonicalize<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<super::path::PathBuf> {
    let path_str = path.as_ref().to_str().ok_or(super::io::ErrorKind::InvalidInput)?;
    let abs_path = if path_str.starts_with('/') {
        path_str.to_string()
    } else {
        match super::env::var("PWD") {
            Ok(cwd) => {
                if cwd == "/" {
                    format!("/{}", path_str)
                } else {
                    format!("{}/{}", cwd, path_str)
                }
            }
            Err(_) => return Err(super::io::ErrorKind::NotFound.into()),
        }
    };
    let mut components: Vec<&str> = Vec::new();
    for component in abs_path.split('/') {
        match component {
            "" | "." => continue,
            ".." => { components.pop(); }
            _ => components.push(component),
        }
    }
    let result = if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", components.join("/"))
    };
    Ok(super::path::PathBuf::from(result))
}

pub fn read_dir<P: AsRef<super::path::Path>>(path: P) -> super::io::Result<ReadDir> {
    let path_str = path.as_ref().to_str().unwrap_or("").to_string();
    let file = File::open(path.as_ref())?;
    Ok(ReadDir { file, _path: path_str, _offset: 0 })
}
