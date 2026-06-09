use core::slice;

use alloc::sync::Arc;

use crate::console::Console;
use crate::fs::{BSIZE, FsError, Inode, Stat};
use crate::log::Operation;
use crate::namespace::{NsProxy, NsType};
use crate::net::ping::PingTable;
use crate::net::tcp::TcpTable;
use crate::net::udp::SocketTable;
use crate::param::{MAXOPBLOCKS, NDEV, NFILE};
use crate::eventfd;
use crate::pipe::Pipe;
use crate::proc;
use crate::sleeplock::SleepLock;
use crate::spinlock::SpinLock;
use crate::syscall::SysError;
use crate::vm::VA;

#[derive(Debug, Clone)]
pub enum FileType {
    None,
    Pipe { pipe: Arc<Pipe> },
    Inode { inode: Inode },
    Device { inode: Inode, major: u16 },
    Socket { socket_id: usize },
    Ping { socket_id: usize },
    TcpSocket { tcp_id: usize },
    Epoll { epoll_id: usize },
    EventFd { eventfd_id: usize },
    MemFd { memfd_id: usize },
    PidFd { pidfd_id: usize },
    Inotify { inotify_id: usize },
    Signalfd { signalfd_id: usize },
    TimerFd { timerfd_id: usize },
    NsFd { ns_proxy: Arc<NsProxy>, nstype: NsType },
}

/// File metadata protected by table-wide spinlock
#[derive(Debug, Clone)]
pub struct FileMeta {
    pub ref_count: usize,
}

/// Per-file mutable state protected by per-file sleeplock
#[derive(Debug, Clone)]
pub struct FileInner {
    pub readable: bool,
    pub writeable: bool,
    pub r#type: FileType,
    pub offset: u32,
    pub nonblocking: bool,
}

pub static FILE_TABLE: FileTable = FileTable::new();

/// Global file table
#[derive(Debug)]
pub struct FileTable {
    /// Protects allocation and reference counts
    pub meta: SpinLock<[FileMeta; NFILE]>,
    /// Per-file locks for concurrent access to different files
    pub inner: [SleepLock<FileInner>; NFILE],
}

impl FileTable {
    const fn new() -> Self {
        let meta = SpinLock::new([const { FileMeta { ref_count: 0 } }; NFILE], "filetable");

        let inner = [const {
            SleepLock::new(
                FileInner {
                    readable: false,
                    writeable: false,
                    r#type: FileType::None,
                    offset: 0,
                    nonblocking: false,
                },
                "file",
            )
        }; NFILE];

        Self { meta, inner }
    }
}

/// File handle, just an index into the `FileTable`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub id: usize,
}

impl File {
    /// Allocates a file structure.
    pub fn alloc() -> Result<Self, FsError> {
        let mut meta = FILE_TABLE.meta.lock();

        for (i, meta) in meta.iter_mut().enumerate() {
            if meta.ref_count == 0 {
                meta.ref_count = 1;

                return Ok(Self { id: i });
            }
        }

        err!(FsError::OutOfFile);
    }

    /// Returns the pipe Arc if this file is a pipe
    pub fn get_pipe(&self) -> Result<alloc::sync::Arc<crate::pipe::Pipe>, SysError> {
        let inner = FILE_TABLE.inner[self.id].lock();
        match &inner.r#type {
            FileType::Pipe { pipe } => Ok(pipe.clone()),
            _ => err!(SysError::BadDescriptor),
        }
    }

    /// Incremets the reference count for the file.
    pub fn dup(&self) -> Self {
        let meta = &mut FILE_TABLE.meta.lock()[self.id];

        assert!(meta.ref_count >= 1, "filedup");

        meta.ref_count += 1;

        self.clone()
    }

    /// Decrements the reference count and closes the file if it reaches 0.
    pub fn close(&mut self) {
        let mut meta_guard = FILE_TABLE.meta.lock();
        let meta = &mut meta_guard[self.id];

        assert!(meta.ref_count >= 1, "fileclose");

        meta.ref_count -= 1;
        if meta.ref_count > 0 {
            return;
        }

        let inner_copy = {
            let mut inner = FILE_TABLE.inner[self.id].lock();
            // copy inner before resetting fields
            let copy = inner.clone();

            meta.ref_count = 0;
            inner.r#type = FileType::None;
            inner.nonblocking = false;

            drop(meta_guard);
            copy
        }; // drop both inner and meta locks

match inner_copy.r#type {
             FileType::None => {}
             FileType::Pipe { pipe } => {
                 pipe.close(inner_copy.writeable);
             }
             FileType::Inode { inode } | FileType::Device { inode, .. } => {
                 let _op = Operation::begin();
                 inode.put();
             }
            FileType::Socket { socket_id } => SocketTable::close(socket_id),
            FileType::TcpSocket { tcp_id } => TcpTable::close(tcp_id),
            FileType::Ping { socket_id } => {
                 crate::net::ping::PingTable::close(socket_id);
             }
            FileType::Epoll { epoll_id } => {
                crate::poll::free_epoll_id(epoll_id);
            }
            FileType::EventFd { eventfd_id } => {
                eventfd::free_eventfd_id(eventfd_id);
            }
            FileType::MemFd { memfd_id } => {
                crate::memfd::free_memfd_id(memfd_id);
            }
            FileType::PidFd { .. } => {}
            FileType::Inotify { inotify_id } => {
                crate::inotify::free_inotify_id(inotify_id);
            }
            FileType::Signalfd { signalfd_id } => {
                crate::signalfd::free_signalfd_id(signalfd_id);
            }
             FileType::TimerFd { timerfd_id } => {
                crate::timerfd::free_timerfd_id(timerfd_id);
            }
             FileType::NsFd { .. } => {}
         }
    }

    /// Gets metadata about file.
    pub fn stat(&self, addr: VA) -> Result<(), SysError> {
        let file_inner = FILE_TABLE.inner[self.id].lock();

        match &file_inner.r#type {
            FileType::Inode { inode } | FileType::Device { inode, .. } => {
                let inode_inner = inode.lock();
                let stat = inode.stat(&inode_inner);
                inode.unlock(inode_inner);

                let src = unsafe {
                    slice::from_raw_parts(&stat as *const _ as *const u8, size_of::<Stat>())
                };
                if log!(proc::copy_to_user(src, addr)).is_err() {
                    err!(SysError::BadAddress);
                }

                Ok(())
            }
            _ => Err(SysError::BadDescriptor),
        }
    }

    /// Reads from file.
    pub fn read(&self, addr: VA, n: usize) -> Result<usize, SysError> {
        let mut file_inner = FILE_TABLE.inner[self.id].lock();

        if !file_inner.readable {
            err!(SysError::BadDescriptor);
        }

        match &mut file_inner.r#type {
            FileType::None => panic!("fileread"),

            FileType::Pipe { pipe } => pipe.read(addr, n),

            FileType::Inode { inode } => {
                let inode = inode.clone();
                let mut inode_inner = inode.lock();

                let dst = unsafe { slice::from_raw_parts_mut(addr.as_mut_ptr(), n) };
                let read = log!(inode.read(&mut inode_inner, file_inner.offset, dst, true));

                if let Ok(read) = read {
                    file_inner.offset += read;
                }

                inode.unlock(inode_inner);

                if let Ok(read) = read {
                    Ok(read as usize)
                } else {
                    err!(SysError::IoError);
                }
            }

            FileType::Device { inode: _, major } => {
                match *major as usize {
                    CONSOLE => (DEVICES[CONSOLE].unwrap().read)(addr, n),
                    CGROUP_DEV => crate::cgroup::device_read(addr, n),
                    _ => match &DEVICES[*major as usize] {
                        Some(dev) => (dev.read)(addr, n),
                        None => err!(SysError::NoEntry),
                    },
                }
            }

            FileType::Socket { socket_id: _ } | FileType::TcpSocket { tcp_id: _ } => {
                // reads from socket should go through recv()
                err!(SysError::BadDescriptor);
            }
            FileType::Ping { socket_id: _ } => {
                err!(SysError::BadDescriptor);
            }
            FileType::Epoll { .. } => {
                err!(SysError::BadDescriptor);
            }
            FileType::EventFd { eventfd_id } => {
                let val = log!(eventfd::eventfd_read(*eventfd_id));
                if let Ok(v) = val {
                    let src = unsafe {
                        core::slice::from_raw_parts(
                            &v as *const _ as *const u8,
                            size_of::<u64>(),
                        )
                    };
                    let copy_n = src.len().min(n);
                    if log!(proc::copy_to_user(src[..copy_n].as_ref(), addr)).is_err() {
                        err!(SysError::BadAddress);
                    }
                    Ok(copy_n)
                } else {
                    Err(val.unwrap_err())
                }
            }
            FileType::MemFd { memfd_id } => {
                let memfd_id = *memfd_id;
                let offset = file_inner.offset as usize;
                let mut buf = alloc::vec![0u8; n];
                drop(file_inner);
                let read_n = log!(crate::memfd::memfd_read(memfd_id, offset, &mut buf));
                if let Ok(n) = read_n {
                    if n > 0 && log!(proc::copy_to_user(&buf[..n], addr)).is_err() {
                        err!(SysError::BadAddress);
                    }
                    let mut inner = FILE_TABLE.inner[self.id].lock();
                    inner.offset += n as u32;
                    Ok(n)
                } else {
                    Err(read_n.unwrap_err())
                }
            }
            FileType::PidFd { .. } => {
                err!(SysError::BadDescriptor);
            }
            FileType::Inotify { inotify_id } => {
                let inotify_id = *inotify_id;
                let mut buf = alloc::vec![0u8; n];
                drop(file_inner);
                let read_n = log!(crate::inotify::inotify_read(inotify_id, &mut buf));
                if let Ok(n) = read_n {
                    if n > 0 && log!(crate::proc::copy_to_user(&buf[..n], addr)).is_err() {
                        err!(SysError::BadAddress);
                    }
                    Ok(n)
                } else {
                    Err(read_n.unwrap_err())
                }
            }
            FileType::Signalfd { signalfd_id } => {
                let signalfd_id = *signalfd_id;
                let mut buf = alloc::vec![0u8; n];
                drop(file_inner);
                let read_n = log!(crate::signalfd::signalfd_read(signalfd_id, &mut buf));
                if let Ok(n) = read_n {
                    if n > 0 && log!(crate::proc::copy_to_user(&buf[..n], addr)).is_err() {
                        err!(SysError::BadAddress);
                    }
                    Ok(n)
                } else {
                    Err(read_n.unwrap_err())
                }
            }
            FileType::TimerFd { timerfd_id } => {
                let timerfd_id = *timerfd_id;
                let nonblock = file_inner.nonblocking;
                drop(file_inner);
                let val = log!(crate::timerfd::timerfd_read(timerfd_id, nonblock));
                if let Ok(v) = val {
                    let val_bytes = v.to_ne_bytes();
                    let copy_len = n.min(val_bytes.len());
                    let mut file_inner = FILE_TABLE.inner[self.id].lock();
                    if log!(crate::proc::copy_to_user(&val_bytes[..copy_len], addr)).is_err() {
                        err!(SysError::BadAddress);
                    }
                    Ok(copy_len)
                } else {
                    Err(val.unwrap_err())
                }
            }
            FileType::NsFd { .. } => err!(SysError::BadDescriptor),
        }
    }

    /// Writes to a file.
    pub fn write(&mut self, addr: VA, n: usize) -> Result<usize, SysError> {
        let mut file_inner = FILE_TABLE.inner[self.id].lock();

        if !file_inner.writeable {
            err!(SysError::BadDescriptor);
        }

        match &mut file_inner.r#type {
            FileType::None => panic!("filewrite"),

            FileType::Pipe { pipe } => pipe.write(addr, n),

            FileType::Inode { inode } => {
                let inode = inode.clone();

                // write a few block at a time to avoid exceeding the maximum log transaction size,
                // including inode, indirect block, allocation blocks, and 2 block of slop for
                // non-aligned writes.
                let max = ((MAXOPBLOCKS - 1 - 1 - 2) / 2) * BSIZE;
                let mut i = 0;

                while i < n {
                    let n1 = (n - i).min(max);

                    let _op = Operation::begin();
                    let mut inode_inner = inode.lock();

                    let src =
                        unsafe { slice::from_raw_parts((addr.as_usize() + i) as *const u8, n1) };
                    let write = log!(inode.write(&mut inode_inner, file_inner.offset, src, true));

                    if let Ok(w) = write {
                        file_inner.offset += w;
                    }

                    inode.unlock(inode_inner);
                    drop(_op);

                    if write.is_err() {
                        break;
                    }

                    i += write.unwrap() as usize;
                }

                if i == n {
                    crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_MODIFY, 0, "");
                    Ok(n)
                } else {
                    err!(SysError::IoError);
                }
            }

            FileType::Device { inode: _, major } => {
                match *major as usize {
                    CONSOLE => (DEVICES[CONSOLE].unwrap().write)(addr, n),
                    CGROUP_DEV => crate::cgroup::device_write(addr, n),
                    _ => match &DEVICES[*major as usize] {
                        Some(dev) => (dev.write)(addr, n),
                        None => err!(SysError::NoEntry),
                    },
                }
            }

            FileType::Socket { socket_id: _ } | FileType::TcpSocket { tcp_id: _ } => {
                // writes to socket should go through send()
                err!(SysError::InvalidArgument);
            }
            FileType::Ping { socket_id: _ } => {
                err!(SysError::InvalidArgument);
            }
            FileType::Epoll { .. } => {
                err!(SysError::InvalidArgument);
            }
            FileType::EventFd { eventfd_id } => {
                let mut val_buf = [0u8; 8];
                if log!(proc::copy_from_user(addr, &mut val_buf)).is_err() {
                    err!(SysError::BadAddress);
                }
                let val = u64::from_ne_bytes(val_buf);
                log!(eventfd::eventfd_write(*eventfd_id, val));
                Ok(8)
            }
            FileType::MemFd { memfd_id } => {
                let memfd_id = *memfd_id;
                let mut buf = alloc::vec![0u8; n];
                if log!(proc::copy_from_user(addr, &mut buf)).is_err() {
                    err!(SysError::BadAddress);
                }
                let offset = file_inner.offset as usize;
                drop(file_inner);
                let write_n = log!(crate::memfd::memfd_write(memfd_id, offset, &buf));
                if let Ok(n) = write_n {
                    let mut inner = FILE_TABLE.inner[self.id].lock();
                    inner.offset += n as u32;
                    Ok(n)
                } else {
                    Err(write_n.unwrap_err())
                }
            }
            FileType::PidFd { .. } => {
                err!(SysError::BadDescriptor);
            }
            FileType::Inotify { .. } => {
                err!(SysError::BadDescriptor);
            }
            FileType::Signalfd { .. } => {
                err!(SysError::BadDescriptor);
            }
            FileType::TimerFd { .. } => {
                err!(SysError::BadDescriptor);
            }
            FileType::NsFd { .. } => err!(SysError::BadDescriptor),
        }
    }

    pub fn ioctl(&self, cmd: usize, arg: usize) -> Result<usize, SysError> {
        let file_inner = FILE_TABLE.inner[self.id].lock();

        match &file_inner.r#type {
            FileType::Device { major, .. } if *major as usize == CONSOLE => {
                Console::ioctl(cmd, arg)
            }
            FileType::Device { major, .. } if *major as usize == CGROUP_DEV => {
                err!(SysError::NotImplemented)
            }
            FileType::Device { .. } => err!(SysError::NotImplemented),

            FileType::Socket { socket_id } => {
                match cmd {
                    Ioctl::SOCKET_GET_PORT => {
                        Ok(SocketTable::get_port_number(*socket_id) as usize)
                    }
                    Ioctl::XV8_VETH_CREATE => {
                        drop(file_inner);
                        crate::net::veth::ioctl_create_veth(VA::new(arg))
                    }
                    _ => err!(SysError::NotImplemented),
                }
            }

            FileType::TcpSocket { tcp_id: _ } => {
                err!(SysError::NotImplemented)
            }

            FileType::Epoll { .. } => err!(SysError::NotImplemented),

            _ => err!(SysError::BadDescriptor),
        }
    }

    pub fn lseek(&self, offset: isize, whence: usize) -> Result<isize, SysError> {
        let mut file_inner = FILE_TABLE.inner[self.id].lock();

         match &file_inner.r#type {
             FileType::None => err!(SysError::BadDescriptor),
                FileType::Pipe { .. } | FileType::Socket { .. } | FileType::Ping { .. } | FileType::TcpSocket { .. } | FileType::Epoll { .. } | FileType::EventFd { .. } | FileType::PidFd { .. } | FileType::Inotify { .. } | FileType::Signalfd { .. } | FileType::TimerFd { .. } | FileType::NsFd { .. } => err!(SysError::IsDirectory),
             FileType::Inode { .. } | FileType::Device { .. } | FileType::MemFd { .. } => {
                let new_offset = match whence {
                    0 => { // SEEK_SET
                        if offset < 0 {
                            err!(SysError::InvalidArgument);
                        }
                        offset as u32
                    }
                    1 => { // SEEK_CUR
                        let base = file_inner.offset as isize;
                        let new = base + offset;
                        if new < 0 {
                            err!(SysError::InvalidArgument);
                        }
                        new as u32
                    }
                    2 => { // SEEK_END
                        match &file_inner.r#type {
                            FileType::Inode { inode } => {
                                let inode_inner = inode.lock();
                                let size = inode_inner.size as isize;
                                let new = size + offset;
                                if new < 0 {
                                    err!(SysError::InvalidArgument);
                                }
                                drop(inode_inner);
                                new as u32
                            }
                            FileType::MemFd { memfd_id } => {
                                let size = crate::memfd::memfd_size(*memfd_id).unwrap_or(0) as isize;
                                let new = size + offset;
                                if new < 0 {
                                    err!(SysError::InvalidArgument);
                                }
                                new as u32
                            }
                            _ => err!(SysError::InvalidArgument),
                        }
                    }
                    _ => err!(SysError::InvalidArgument),
                };
                file_inner.offset = new_offset;
                Ok(new_offset as isize)
            }
        }
    }

    pub fn truncate(&self, _length: usize) -> Result<(), SysError> {
        let file_inner = FILE_TABLE.inner[self.id].lock();

        match &file_inner.r#type {
            FileType::Inode { inode } => {
                let mut inode = inode.clone();
                let mut inode_inner = inode.lock();
                inode.trunc(&mut inode_inner);
                inode.unlock(inode_inner);
                Ok(())
            }
            FileType::Epoll { .. } => err!(SysError::BadDescriptor),
            _ => err!(SysError::BadDescriptor),
        }
    }

    pub fn chmod(&self, mode: u16) -> Result<(), SysError> {
        let file_inner = FILE_TABLE.inner[self.id].lock();

        match &file_inner.r#type {
            FileType::Inode { inode } | FileType::Device { inode, .. } => {
                let inode = inode.clone();
                let mut inode_inner = inode.lock();
                inode_inner.mode = (inode_inner.mode & !0o777) | (mode & 0o777);
                inode.update(&inode_inner);
                inode.unlock(inode_inner);
                crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_ATTRIB, 0, "");
                Ok(())
            }
            FileType::Epoll { .. } => err!(SysError::BadDescriptor),
            _ => err!(SysError::BadDescriptor),
        }
    }

    pub fn chown(&self, uid: u16, gid: u16) -> Result<(), SysError> {
        let file_inner = FILE_TABLE.inner[self.id].lock();

        match &file_inner.r#type {
            FileType::Inode { inode } | FileType::Device { inode, .. } => {
                let inode = inode.clone();
                let mut inode_inner = inode.lock();
                inode_inner.uid = uid;
                inode_inner.gid = gid;
                inode.update(&inode_inner);
                inode.unlock(inode_inner);
                crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_ATTRIB, 0, "");
                Ok(())
            }
            FileType::Epoll { .. } => err!(SysError::BadDescriptor),
            _ => err!(SysError::BadDescriptor),
        }
    }
}

/// File open flags (POSIX standard)
pub struct OpenFlag;

impl OpenFlag {
    pub const READ_ONLY: usize = 0x000;
    pub const WRITE_ONLY: usize = 0x001;
    pub const READ_WRITE: usize = 0x002;
    pub const CREATE: usize = 0x040;
    pub const EXCLUSIVE: usize = 0x080;
    pub const TRUNCATE: usize = 0x200;
    pub const APPEND: usize = 0x400;
    pub const NON_BLOCK: usize = 0x800;
}

/// Device interface
#[derive(Debug, Clone, Copy)]
pub struct Device {
    pub read: fn(addr: VA, n: usize) -> Result<usize, SysError>,
    pub write: fn(addr: VA, n: usize) -> Result<usize, SysError>,
}

/// Device-specific ioctl commands
pub struct Ioctl;

impl Ioctl {
    pub const CONSOLE_SET_RAW: usize = 1;
    pub const CONSOLE_SET_FG_PID: usize = 2;

    pub const SOCKET_GET_PORT: usize = 3;
    pub const XV8_VETH_CREATE: usize = 100;
}

/// Console device major number
pub const CONSOLE: usize = 1;

/// Cgroup device major number
pub const CGROUP_DEV: usize = 2;

/// Device table
pub static DEVICES: [Option<Device>; NDEV] = {
    let mut devices = [None; NDEV];
    devices[CONSOLE] = Some(Device {
        read: Console::read,
        write: Console::write,
    });
    devices
};
