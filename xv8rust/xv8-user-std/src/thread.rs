use core::sync::atomic::{AtomicU32, Ordering};
use core::str;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::ffi::CString;

use super::io::{self, ErrorKind};
use super::time::Duration;

const CLONE_VM: usize = 0x100;
const CLONE_SIGHAND: usize = 0x80000;
const CLONE_SETTLS: usize = 0x800;
const DEFAULT_STACK_SIZE: usize = 0x4000;
const STACK_GUARD: usize = 256;

const FUTEX_WAIT: u32 = 0;
const FUTEX_WAKE: u32 = 1;

const EMPTY: u32 = 0;
const NOTIFIED: u32 = 1;
const PARKED: u32 = 2;

#[repr(C)]
struct Tcb {
    park: AtomicU32,
    name: usize,
    args_ptr: usize,
}

struct Args<F, T> {
    f: Option<F>,
    result: Option<T>,
    join_w: usize,
}

static MAIN_PARK: AtomicU32 = AtomicU32::new(EMPTY);

fn current_park_addr() -> usize {
    let tp: usize;
    unsafe { core::arch::asm!("mv {}, tp", out(reg) tp) };
    if tp == 0 {
        &MAIN_PARK as *const AtomicU32 as usize
    } else {
        tp
    }
}

fn current_name() -> Option<String> {
    let tp: usize;
    unsafe { core::arch::asm!("mv {}, tp", out(reg) tp) };
    if tp == 0 {
        return None;
    }
    unsafe {
        let tcb = &*(tp as *const Tcb);
        if tcb.name == 0 {
            None
        } else {
            let cstr = core::ffi::CStr::from_ptr(tcb.name as *const core::ffi::c_char);
            Some(cstr.to_str().ok()?.to_string())
        }
    }
}

pub struct Builder {
    name: Option<String>,
    stack_size: usize,
}

impl Builder {
    pub fn new() -> Builder {
        Builder { name: None, stack_size: DEFAULT_STACK_SIZE }
    }

    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    pub fn stack_size(mut self, size: usize) -> Builder {
        self.stack_size = size;
        self
    }

    pub fn spawn<F, T>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let mut join_fds = [0usize; 2];
        if xv8_libc::pipe(join_fds.as_mut_ptr()) < 0 {
            return Err(io::Error::new(ErrorKind::Other, "spawn: pipe failed"));
        }
        let join_r = join_fds[0];
        let join_w = join_fds[1];

        let cname = self.name.as_ref().and_then(|n| CString::new(n.as_str()).ok());
        let name_ptr = cname.as_ref().map(|c| c.as_ptr() as usize).unwrap_or(0);

        let args = Box::new(Args {
            f: Some(f),
            result: None,
            join_w,
        });
        let args_ptr = Box::into_raw(args);

        let tcb = Box::new(Tcb {
            park: AtomicU32::new(EMPTY),
            name: name_ptr,
            args_ptr: args_ptr as usize,
        });
        let tcb_ptr = Box::into_raw(tcb);
        let park_addr = tcb_ptr as usize;

        if cname.is_some() {
            // leak CString so the pointer remains valid in child
            core::mem::forget(cname);
        }

        let stack_base = xv8_libc::sbrk(self.stack_size as isize);
        if stack_base < 0 {
            let _ = unsafe { Box::from_raw(args_ptr) };
            let _ = unsafe { Box::from_raw(tcb_ptr) };
            let _ = xv8_libc::close(join_r);
            let _ = xv8_libc::close(join_w);
            return Err(io::Error::new(ErrorKind::Other, "spawn: sbrk failed"));
        }
        let stack_top = stack_base as usize + self.stack_size - STACK_GUARD;

        let flags = CLONE_VM | CLONE_SIGHAND | CLONE_SETTLS;
        let tid = xv8_libc::clone_tls(flags, stack_top, 0, tcb_ptr as usize);

        if tid < 0 {
            let _ = unsafe { Box::from_raw(args_ptr) };
            let _ = unsafe { Box::from_raw(tcb_ptr) };
            let _ = xv8_libc::close(join_r);
            let _ = xv8_libc::close(join_w);
            return Err(io::Error::new(ErrorKind::Other, "spawn: clone failed"));
        }

        if tid == 0 {
            child_entry::<F, T>(tcb_ptr)
        }

        let _ = xv8_libc::close(join_w);

        Ok(JoinHandle {
            thread: Thread {
                id: tid as usize,
                join_fd: join_r,
                park_addr,
                name: self.name,
            },
            result_ptr: unsafe { &mut (*args_ptr).result as *mut Option<T> },
        })
    }
}

impl Default for Builder {
    fn default() -> Builder {
        Builder::new()
    }
}

pub struct Thread {
    id: usize,
    join_fd: usize,
    park_addr: usize,
    name: Option<String>,
}

impl Thread {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn unpark(&self) {
        let state = unsafe { &*(self.park_addr as *const AtomicU32) };
        if state.swap(NOTIFIED, Ordering::Release) == PARKED {
            xv8_libc::futex(self.park_addr as *const u32, FUTEX_WAKE, 1);
        }
    }
}

pub fn current() -> Thread {
    let id = xv8_libc::gettid() as usize;
    Thread { id, join_fd: 0, park_addr: current_park_addr(), name: current_name() }
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    Builder::new().spawn(f).expect("spawn failed")
}

#[inline(never)]
fn child_entry<F: FnOnce() -> T, T>(tcb_ptr: *mut Tcb) -> ! {
    let args = unsafe { &mut *((*tcb_ptr).args_ptr as *mut Args<F, T>) };
    let f = args.f.take().unwrap();
    let result = f();
    args.result = Some(result);
    let _ = xv8_libc::write(args.join_w, &[1u8] as *const u8, 1);
    xv8_libc::exit(0);
}

pub fn park() {
    let addr = current_park_addr();
    let state = unsafe { &*(addr as *const AtomicU32) };

    if state.swap(EMPTY, Ordering::Acquire) == NOTIFIED {
        return;
    }

    while state.compare_exchange(EMPTY, PARKED, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
        xv8_libc::futex(addr as *const u32, FUTEX_WAIT, PARKED);
        if state.swap(EMPTY, Ordering::Acquire) == NOTIFIED {
            return;
        }
    }

    state.swap(EMPTY, Ordering::Acquire);
}

pub fn unpark(t: &Thread) {
    t.unpark();
}

pub fn yield_now() {
    let _ = xv8_libc::sleep(0);
}

pub fn sleep(dur: Duration) {
    if dur.secs == 0 && dur.nanos == 0 {
        return;
    }
    let mut ticks = dur.secs.saturating_mul(100);
    if dur.nanos != 0 {
        ticks = ticks.saturating_add((dur.nanos as u64 + 9_999_999) / 10_000_000);
    }
    let _ = xv8_libc::sleep(ticks as usize);
}

pub fn available_parallelism() -> core::num::NonZeroUsize {
    core::num::NonZeroUsize::new(1).unwrap()
}

pub struct JoinHandle<T> {
    thread: Thread,
    result_ptr: *mut Option<T>,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> T {
        let mut buf = [0u8; 1];
        let n = xv8_libc::read(self.thread.join_fd, buf.as_mut_ptr(), 1);
        assert!(n > 0, "join: pipe read failed");
        let _ = xv8_libc::close(self.thread.join_fd);
        let result = unsafe { Box::from_raw(self.result_ptr) };
        result.unwrap()
    }

    pub fn thread(&self) -> &Thread {
        &self.thread
    }
}
