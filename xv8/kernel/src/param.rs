/// maximum number of CPUs
pub const NCPU: usize = 8;

/// maximum number of processes
pub const NPROC: usize = 128;

/// open files per process
pub const NOFILE: usize = 256;

/// open files per system
pub const NFILE: usize = 100;

/// maximum number of active inodes
pub const NINODE: usize = 50;

/// maximum major device number
pub const NDEV: usize = 10;

/// device nubmer of file system root disk
pub const ROOTDEV: u32 = 1;

/// open sockets per system
pub const NSOCKET: usize = 16;

/// open ping sockets per system
pub const NPING: usize = 16;

/// open TCP sockets per system
pub const NTCP: usize = 16;

/// max exec arguments
pub const MAXARG: usize = 32;

/// max # of blocks any FS op writes
pub const MAXOPBLOCKS: usize = 10;

/// max data blocks in on-disk log
pub const LOGBLOCKS: usize = MAXOPBLOCKS * 3;

/// size of disk block cache
pub const NBUF: usize = MAXOPBLOCKS * 3;

/// maximum file path name
pub const MAXPATH: usize = 128;

/// kernel stack pages per process (debug needs more stack to dump)
#[cfg(debug_assertions)]
pub const NKSTACK_PAGES: usize = 8;
#[cfg(not(debug_assertions))]
pub const NKSTACK_PAGES: usize = 1;

/// user stack pages
pub const USERSTACK: usize = 4;

/// max epoll instances per system
pub const NEPOLL: usize = 16;

/// max eventfd instances per system
pub const NEVENTFD: usize = 16;
pub const NMEMFD: usize = 16;
pub const NPIDFD: usize = 16;
pub const NINOTIFY: usize = 16;
pub const NINOTIFY_WATCHES: usize = 64;
pub const NSIGNALFD: usize = 16;
pub const NTIMERFD: usize = 16;

/// max mmap regions per process
pub const MMAP_REGIONS: usize = 16;

/// mmap base address (below trapframe/trampoline)
pub const MMAP_BASE: usize = crate::memlayout::TRAPFRAME - (256 * 1024 * 1024);
