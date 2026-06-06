use core::task::Waker;
use xv8_libc;
use xv8_user_std::collections::HashMap;
use xv8_user_std::sync::Mutex;

fn lock<'a, T>(mutex: &'a Mutex<T>) -> xv8_user_std::sync::MutexGuard<'a, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

static REACTOR: Mutex<Option<ReactorInner>> = Mutex::new(None);

struct ReactorInner {
    epoll_fd: usize,
    wakers: HashMap<usize, Waker>,
}

pub fn init() -> Option<usize> {
    let mut guard = lock(&REACTOR);
    if guard.is_some() {
        return guard.as_ref().map(|r| r.epoll_fd);
    }
    let ret = xv8_libc::epoll_create1(0);
    if ret < 0 {
        return None;
    }
    let epoll_fd = ret as usize;
    *guard = Some(ReactorInner {
        epoll_fd,
        wakers: HashMap::new(),
    });
    Some(epoll_fd)
}

pub fn ensure_init() {
    init();
}

pub fn register_read(fd: usize, waker: Waker) {
    let mut reactor = lock(&REACTOR);
    if let Some(ref mut r) = *reactor {
        let event = xv8_libc::EpollEvent { events: xv8_libc::EPOLLIN, data: fd as u64 };
        let ret = xv8_libc::epoll_ctl(r.epoll_fd, xv8_libc::EPOLL_CTL_ADD, fd, &event as *const _);
        if ret < 0 {
            let ret = xv8_libc::epoll_ctl(r.epoll_fd, xv8_libc::EPOLL_CTL_MOD, fd, &event as *const _);
            let _ = ret;
        }
        r.wakers.insert(fd, waker);
    }
}

pub fn register_write(fd: usize, waker: Waker) {
    let mut reactor = lock(&REACTOR);
    if let Some(ref mut r) = *reactor {
        let event = xv8_libc::EpollEvent { events: xv8_libc::EPOLLOUT, data: fd as u64 };
        let ret = xv8_libc::epoll_ctl(r.epoll_fd, xv8_libc::EPOLL_CTL_ADD, fd, &event as *const _);
        if ret < 0 {
            let ret = xv8_libc::epoll_ctl(r.epoll_fd, xv8_libc::EPOLL_CTL_MOD, fd, &event as *const _);
            let _ = ret;
        }
        r.wakers.insert(fd, waker);
    }
}

pub fn unregister(fd: usize) {
    let mut reactor = lock(&REACTOR);
    if let Some(ref mut r) = *reactor {
        r.wakers.remove(&fd);
        let _ = xv8_libc::epoll_ctl(r.epoll_fd, xv8_libc::EPOLL_CTL_DEL, fd, core::ptr::null());
    }
}

pub fn poll_events(timeout: isize) {
    let mut events = [xv8_libc::EpollEvent { events: 0, data: 0 }; 64];
    let epoll_fd = {
        let reactor = lock(&REACTOR);
        reactor.as_ref().map(|r| r.epoll_fd)
    };
    let epoll_fd = match epoll_fd {
        Some(fd) => fd,
        None => return,
    };
    let n = xv8_libc::epoll_wait(epoll_fd, events.as_mut_ptr(), 64, timeout);
    if n <= 0 {
        return;
    }
    let mut reactor = lock(&REACTOR);
    if let Some(ref mut r) = *reactor {
        for i in 0..n as usize {
            let fd = events[i].data as usize;
            if let Some(waker) = r.wakers.remove(&fd) {
                waker.wake();
            }
        }
    }
}