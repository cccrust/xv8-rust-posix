use crate::fs;

pub trait MetadataExt {
    fn mode(&self) -> u32;
    fn uid(&self) -> u32;
    fn gid(&self) -> u32;
    fn size(&self) -> u64;
    fn mtime(&self) -> i64;
    fn nlink(&self) -> u32;
}

pub trait PermissionsExt {
    fn mode(&self) -> u32;
    fn set_mode(&mut self, mode: u32);
}

impl MetadataExt for fs::Metadata {
    fn mode(&self) -> u32 { self.mode }
    fn uid(&self) -> u32 { self.uid }
    fn gid(&self) -> u32 { self.gid }
    fn size(&self) -> u64 { self.size }
    fn mtime(&self) -> i64 { self.mtime }
    fn nlink(&self) -> u32 { self.nlink }
}

impl PermissionsExt for fs::Permissions {
    fn mode(&self) -> u32 { self.mode }
    fn set_mode(&mut self, mode: u32) { self.mode = mode; }
}

pub fn symlink<P: AsRef<crate::path::Path>, Q: AsRef<crate::path::Path>>(src: P, dst: Q) -> crate::io::Result<()> {
    let src_str = src.as_ref().to_str().ok_or(crate::io::ErrorKind::InvalidInput)?;
    let dst_str = dst.as_ref().to_str().ok_or(crate::io::ErrorKind::InvalidInput)?;
    let c_src = crate::ffi::CString::new(src_str).map_err(|_| crate::io::ErrorKind::InvalidInput)?;
    let c_dst = crate::ffi::CString::new(dst_str).map_err(|_| crate::io::ErrorKind::InvalidInput)?;
    let ret = xv8_libc::symlink(c_src.as_ptr() as *const u8, c_dst.as_ptr() as *const u8);
    if ret < 0 { Err(crate::io::ErrorKind::Other.into()) } else { Ok(()) }
}
