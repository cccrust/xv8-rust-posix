use alloc::vec;
use alloc::vec::Vec;

use crate::file::{FILE_TABLE, FileType};
use crate::net::tcp::{self, TcpTable};
use crate::param::NEPOLL;
use crate::proc::{self, Channel};
use crate::spinlock::SpinLock;
use crate::syscall::{SysError, SyscallArgs};
use crate::sysfile::fd_alloc;

pub const POLLIN: u16 = 0x001;
pub const POLLOUT: u16 = 0x004;
pub const POLLERR: u16 = 0x008;
pub const POLLHUP: u16 = 0x010;

pub const EPOLL_CTL_ADD: usize = 1;
pub const EPOLL_CTL_DEL: usize = 2;
pub const EPOLL_CTL_MOD: usize = 3;
pub const EPOLLIN: u32 = 0x001;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008;
pub const EPOLLHUP: u32 = 0x010;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EpollEvent {
    pub events: u32,
    pub data: u64,
}

#[derive(Debug, Clone)]
pub struct EpollEntry {
    pub fd: usize,
    pub events: u32,
    pub data: u64,
    pub tcp_id: Option<usize>,
}

#[derive(Debug)]
pub struct EpollInstance {
    pub entries: Vec<EpollEntry>,
    pub triggered: Vec<EpollEvent>,
    pub waiting: bool,
}

pub struct EpollTable {
    pub entries: [Option<EpollInstance>; NEPOLL],
    pub next_id: usize,
}

static EPOLL_TABLE: SpinLock<EpollTable> = SpinLock::new(
    EpollTable {
        entries: [const { None }; NEPOLL],
        next_id: 0,
    },
    "epoll_table",
);

fn alloc_epoll_id() -> Result<usize, SysError> {
    let mut table = EPOLL_TABLE.lock();
    for i in 0..table.entries.len() {
        if table.entries[i].is_none() {
            table.entries[i] = Some(EpollInstance {
                entries: Vec::new(),
                triggered: Vec::new(),
                waiting: false,
            });
            return Ok(i);
        }
    }
    err!(SysError::FileTableFull)
}

pub(crate) fn free_epoll_id(id: usize) {
    let mut table = EPOLL_TABLE.lock();
    if id < table.entries.len() {
        table.entries[id] = None;
    }
}

fn fd_readiness(fd: usize) -> (bool, bool) {
    if fd >= crate::param::NOFILE {
        return (false, false);
    }
    let (_proc, data) = proc::current_proc_and_data_mut();
    let file = {
        let files = data.open_files.as_ref().unwrap().files.lock();
        match &files[fd] {
            Some(f) => f.clone(),
            None => return (false, false),
        }
    };

    let inner = FILE_TABLE.inner[file.id].lock();
    match &inner.r#type {
        FileType::TcpSocket { tcp_id } => {
            let (r, w) = tcp::tcp_readiness(*tcp_id);
            (r, w)
        }
        FileType::Pipe { pipe } => {
            let readable = pipe.readable();
            let writeable = pipe.writeable();
            (readable, writeable)
        }
        FileType::Inode { .. } | FileType::Device { .. } => {
            (inner.readable, inner.writeable)
        }
        FileType::Socket { .. } | FileType::Ping { .. } => {
            (inner.readable, inner.writeable)
        }
        FileType::Epoll { .. } => (false, false),
        FileType::EventFd { eventfd_id } => {
            crate::eventfd::eventfd_readiness(*eventfd_id)
        }
        FileType::MemFd { .. } => (true, true),
        FileType::PidFd { pidfd_id } => {
            let alive = crate::pidfd::pidfd_is_alive(*pidfd_id);
            (!alive, false)
        }
        FileType::Inotify { inotify_id } => {
            crate::inotify::inotify_readiness(*inotify_id)
        }
        FileType::Signalfd { signalfd_id } => {
            crate::signalfd::signalfd_readiness(*signalfd_id)
        }
        FileType::TimerFd { timerfd_id } => {
            crate::timerfd::timerfd_readiness(*timerfd_id)
        }
        FileType::None => (false, false),
        FileType::NsFd { .. } => (false, false),
    }
}

fn find_tcp_id(fd: usize) -> Option<usize> {
    let (_proc, data) = proc::current_proc_and_data_mut();
    let file = {
        let files = data.open_files.as_ref().unwrap().files.lock();
        files[fd].as_ref()?.clone()
    };
    let inner = FILE_TABLE.inner[file.id].lock();
    match &inner.r#type {
        FileType::TcpSocket { tcp_id } => Some(*tcp_id),
        _ => None,
    }
}

pub fn sys_poll(args: &SyscallArgs) -> Result<usize, SysError> {
    let fds_addr = args.get_addr(0);
    let nfds = args.get_int(1) as usize;
    let _timeout = args.get_int(2);

    if nfds == 0 {
        return Ok(0);
    }

    let (_proc, data) = proc::current_proc_and_data_mut();
    let pt = data.pagetable_mut();

    let mut poll_fds = vec![PollFd { fd: 0, events: 0, revents: 0 }; nfds];
    let fds_bytes = unsafe {
        core::slice::from_raw_parts_mut(
            poll_fds.as_mut_ptr() as *mut u8,
            nfds * size_of::<PollFd>(),
        )
    };
    if pt.copy_from(fds_addr, fds_bytes).is_err() {
        err!(SysError::BadAddress);
    }

    let mut ready = 0;
    for poll_fd in poll_fds.iter_mut() {
        poll_fd.revents = 0;
        if poll_fd.fd < 0 {
            continue;
        }
        let fd = poll_fd.fd as usize;
        let events = poll_fd.events as u16;
        let (readable, writable) = fd_readiness(fd);
        if (events & POLLIN) != 0 && readable {
            poll_fd.revents |= POLLIN as i16;
            ready += 1;
        }
        if (events & POLLOUT) != 0 && writable {
            poll_fd.revents |= POLLOUT as i16;
            ready += 1;
        }
    }

    if pt.copy_to(fds_bytes, fds_addr).is_err() {
        err!(SysError::BadAddress);
    }

    Ok(ready)
}

pub fn sys_epoll_create1(args: &SyscallArgs) -> Result<usize, SysError> {
    let _flags = args.get_int(0) as usize;

    let epoll_id = try_log!(alloc_epoll_id());

    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(fd_alloc(file.clone()));

    let mut inner = FILE_TABLE.inner[file.id].lock();
    inner.r#type = FileType::Epoll { epoll_id };

    Ok(fd)
}

pub fn sys_epoll_ctl(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, epoll_file) = try_log!(args.get_file(0));
    let op = args.get_int(1) as usize;
    let target_fd = args.get_int(2) as usize;
    let event_addr = args.get_addr(3);

    let epoll_id = {
        let inner = FILE_TABLE.inner[epoll_file.id].lock();
        let FileType::Epoll { epoll_id } = inner.r#type else {
            err!(SysError::BadDescriptor)
        };
        epoll_id
    };

    let mut event = EpollEvent { events: 0, data: 0 };
    if event_addr.as_usize() != 0 && op != EPOLL_CTL_DEL {
        let (_proc, data) = proc::current_proc_and_data_mut();
        let pt = data.pagetable_mut();
        let event_bytes = unsafe {
            core::slice::from_raw_parts_mut(
                &mut event as *mut _ as *mut u8,
                size_of::<EpollEvent>(),
            )
        };
        if pt.copy_from(event_addr, event_bytes).is_err() {
            err!(SysError::BadAddress);
        }
    }

    let mut table = EPOLL_TABLE.lock();
    let Some(ref mut instance) = table.entries[epoll_id] else {
        err!(SysError::BadDescriptor)
    };

    match op {
        EPOLL_CTL_ADD => {
            if instance.entries.iter().any(|e| e.fd == target_fd) {
                err!(SysError::AlreadyExists)
            }
            let tcp_id = find_tcp_id(target_fd);
            if tcp_id.is_some() {
                let mut tcp_table = crate::net::tcp::TCP_TABLE.lock();
                if let Some(ref mut conn) = tcp_table.entries[tcp_id.unwrap()] {
                    conn.epoll_instances.push(epoll_id);
                }
            }
            instance.entries.push(EpollEntry {
                fd: target_fd,
                events: event.events,
                data: event.data,
                tcp_id,
            });
        }
        EPOLL_CTL_DEL => {
            if let Some(pos) = instance.entries.iter().position(|e| e.fd == target_fd) {
                let removed = instance.entries.remove(pos);
                if let Some(tcp_id) = removed.tcp_id {
                    let mut tcp_table = crate::net::tcp::TCP_TABLE.lock();
                    if let Some(ref mut conn) = tcp_table.entries[tcp_id] {
                        conn.epoll_instances.retain(|&id| id != epoll_id);
                    }
                }
            }
        }
        EPOLL_CTL_MOD => {
            if let Some(entry) = instance.entries.iter_mut().find(|e| e.fd == target_fd) {
                entry.events = event.events;
                entry.data = event.data;
            }
        }
        _ => err!(SysError::InvalidArgument),
    }

    Ok(0)
}

pub fn sys_epoll_wait(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, epoll_file) = try_log!(args.get_file(0));
    let events_addr = args.get_addr(1);
    let max_events = args.get_int(2) as usize;
    let timeout = args.get_int(3);

    let epoll_id = {
        let inner = FILE_TABLE.inner[epoll_file.id].lock();
        let FileType::Epoll { epoll_id } = inner.r#type else {
            err!(SysError::BadDescriptor)
        };
        epoll_id
    };

    loop {
        if proc::current_proc().is_killed() {
            err!(SysError::Interrupted);
        }

        let mut triggered = {
            let mut table = EPOLL_TABLE.lock();
            let Some(ref mut instance) = table.entries[epoll_id] else {
                err!(SysError::BadDescriptor);
            };

            let mut ready = Vec::new();
            for entry in &instance.entries {
                let (readable, writable) = fd_readiness(entry.fd);
                let mut revents: u32 = 0;
                if (entry.events & EPOLLIN) != 0 && readable {
                    revents |= EPOLLIN;
                }
                if (entry.events & EPOLLOUT) != 0 && writable {
                    revents |= EPOLLOUT;
                }
                if revents != 0 {
                    ready.push(EpollEvent {
                        events: revents,
                        data: entry.data,
                    });
                    if ready.len() >= max_events {
                        break;
                    }
                }
            }

            if !ready.is_empty() || timeout == 0 {
                drop(table);
                let n = ready.len().min(max_events);
                let (_proc, data) = proc::current_proc_and_data_mut();
                let pt = data.pagetable_mut();
                let events_bytes = unsafe {
                    core::slice::from_raw_parts(
                        ready.as_ptr() as *const u8,
                        n * size_of::<EpollEvent>(),
                    )
                };
                if pt.copy_to(events_bytes, events_addr).is_err() {
                    err!(SysError::BadAddress);
                }
                return Ok(n);
            }

            instance.waiting = true;
            let table_guard = table;
            table = proc::sleep(Channel::Epoll(epoll_id), table_guard);
            let Some(ref mut inst) = table.entries[epoll_id] else {
                err!(SysError::BadDescriptor);
            };
            inst.waiting = false;

            if !inst.triggered.is_empty() {
                // Clear triggered events and fall through to loop back.
                // fd_readiness will report the correct events with proper
                // entry.data (the triggered events have data=0 which would
                // confuse userspace).
                inst.triggered.clear();
            }
        };
    }
}

pub fn epoll_notify_instances(epfd: usize, events: u32) {
    let mut table = EPOLL_TABLE.lock();
    if let Some(ref mut instance) = table.entries[epfd] {
        instance.triggered.push(EpollEvent {
            events,
            data: 0,
        });
        let waiting = instance.waiting;
        drop(table);
        if waiting {
            proc::wakeup(Channel::Epoll(epfd));
        }
    }
}