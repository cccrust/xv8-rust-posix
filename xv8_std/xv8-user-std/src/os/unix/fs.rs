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
