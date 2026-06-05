use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

extern "Rust" {
    fn main() -> ();
}

struct Xv8Alloc;

unsafe impl GlobalAlloc for Xv8Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align());
        let ptr = xv8_libc::sbrk(size as isize);
        if ptr < 0 { core::ptr::null_mut() } else { ptr as *mut u8 }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: Xv8Alloc = Xv8Alloc;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    xv8_libc::exit(1)
}

pub trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 { 0 }
}

impl Termination for i32 {
    fn report(self) -> i32 { self }
}

fn safe_main() {
    unsafe { main() }
}

#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
    _sigpipe: u8,
) -> isize {
    let code = main().report();
    if code != 0 {
        xv8_libc::exit(code as usize);
    }
    0
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start(argc: isize, argv: *const *const u8) -> ! {
    xv8_libc::args::init(argc as usize, argv);
    lang_start(safe_main, argc, argv, 0);
    xv8_libc::exit(0)
}
