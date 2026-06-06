use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;

use crate::file::File;
use crate::param::NOFILE;
use crate::spinlock::SpinLock;

pub struct FdTable {
    pub files: SpinLock<Box<[Option<File>; NOFILE]>>,
}

impl FdTable {
    pub fn new() -> Self {
        // Use vec to allocate the array on the heap, avoiding 2KB on the kernel stack
        let v = vec![None; NOFILE];
        let arr: Box<[Option<File>; NOFILE]> = v.into_boxed_slice().try_into().unwrap();
        Self {
            files: SpinLock::new(arr, "fd_table"),
        }
    }

    pub fn alloc_empty() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub fn dup_from(other: &Arc<Self>) -> Arc<Self> {
        let src = other.files.lock();
        let new_table = Self::new();
        let mut dst = new_table.files.lock();
        for (i, entry) in src.iter().enumerate() {
            if let Some(file) = entry {
                dst[i] = Some(file.dup());
            }
        }
        drop(src);
        drop(dst);
        Arc::new(new_table)
    }
}
