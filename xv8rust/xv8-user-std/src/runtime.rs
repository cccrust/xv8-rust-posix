use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};

extern "Rust" {
    fn main() -> ();
}

const CHUNK_SIZE: usize = 65536;

struct ChunkAlloc {
    base: AtomicUsize,
    offset: AtomicUsize,
}

unsafe impl GlobalAlloc for ChunkAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align());
        let align = layout.align().max(16);
        loop {
            let base = self.base.load(Ordering::Relaxed);
            if base == 0 {
                let ptr = xv8_libc::sbrk(CHUNK_SIZE as isize);
                if ptr < 0 { return core::ptr::null_mut(); }
                self.base.store(ptr as usize, Ordering::Release);
                self.offset.store(0, Ordering::Release);
                continue;
            }
            let off = self.offset.load(Ordering::Relaxed);
            let misalign = off % align;
            let start = if misalign == 0 { off } else { off + align - misalign };
            let end = start + size;
            if end <= CHUNK_SIZE {
                if self.offset.compare_exchange(off, end, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
                    return (base + start) as *mut u8;
                }
            } else {
                let ptr = xv8_libc::sbrk(CHUNK_SIZE as isize);
                if ptr < 0 { return core::ptr::null_mut(); }
                self.base.store(ptr as usize, Ordering::Release);
                self.offset.store(size, Ordering::Release);
                return (ptr as usize) as *mut u8;
            }
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: ChunkAlloc = ChunkAlloc {
    base: AtomicUsize::new(0),
    offset: AtomicUsize::new(0),
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::panic::run_hook(info);
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
