use alloc::boxed::Box;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

static HOOK_SET: AtomicBool = AtomicBool::new(false);

type PanicHook = Box<dyn Fn(&PanicInfo) + 'static>;

static mut HOOK: Option<PanicHook> = None;

pub fn set_hook(hook: Box<dyn Fn(&PanicInfo) + 'static>) {
    unsafe { HOOK = Some(hook); }
    HOOK_SET.store(true, Ordering::Release);
}

pub fn take_hook() -> Box<dyn Fn(&PanicInfo) + 'static> {
    HOOK_SET.store(false, Ordering::Release);
    unsafe {
        HOOK.take().unwrap_or_else(|| Box::new(|_| {}))
    }
}

pub(crate) fn run_hook(info: &PanicInfo) {
    if HOOK_SET.load(Ordering::Acquire) {
        unsafe {
            if let Some(ref hook) = HOOK {
                hook(info);
            }
        }
    }
}
