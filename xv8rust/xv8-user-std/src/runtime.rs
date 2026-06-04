use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use core::ptr;

extern "Rust" {
    fn main() -> ();
}

struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

const HEADER_SIZE: usize = core::mem::size_of::<usize>(); // just store size before each block

struct Xv8Alloc {
    free_list: core::sync::atomic::AtomicPtr<FreeBlock>,
}

unsafe impl GlobalAlloc for Xv8Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align());
        let needed = size + HEADER_SIZE;

        let mut prev: *mut FreeBlock = ptr::null_mut();
        let mut curr = self.free_list.load(core::sync::atomic::Ordering::Acquire);
        while !curr.is_null() {
            let block_size = (*curr).size;
            if block_size >= needed {
                // Remove from free list
                if prev.is_null() {
                    self.free_list.store((*curr).next, core::sync::atomic::Ordering::Relaxed);
                } else {
                    (*prev).next = (*curr).next;
                }

                let alloc_size;
                if block_size - needed >= HEADER_SIZE + core::mem::size_of::<usize>() {
                    // Split: keep remaining as free block
                    alloc_size = needed;
                    let leftover_addr = (curr as *mut u8).add(needed);
                    let leftover = leftover_addr as *mut FreeBlock;
                    let head = self.free_list.load(core::sync::atomic::Ordering::Relaxed);
                    ptr::write(leftover, FreeBlock { size: block_size - needed, next: head });
                    self.free_list.store(leftover, core::sync::atomic::Ordering::Release);
                } else {
                    alloc_size = block_size;
                }

                *(curr as *mut usize) = alloc_size;
                return (curr as *mut u8).add(HEADER_SIZE);
            }
            prev = curr;
            curr = (*curr).next;
        }

        // No suitable free block: grow heap
        let brk = xv8_libc::sbrk(needed as isize);
        if brk < 0 { return ptr::null_mut(); }
        *(brk as *mut usize) = needed;
        (brk as *mut u8).add(HEADER_SIZE)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() { return; }
        let header = (ptr as *mut u8).sub(HEADER_SIZE) as *mut usize;
        let size = *header;
        let block = header as *mut FreeBlock;
        ptr::write(block, FreeBlock { size, next: ptr::null_mut() });

        // Insert sorted by address for coalescing
        let curr = self.free_list.load(core::sync::atomic::Ordering::Acquire);
        if curr.is_null() || (curr as usize) > (block as usize) {
            (*block).next = curr;
            self.free_list.store(block, core::sync::atomic::Ordering::Release);
            if !curr.is_null() && ((block as *mut u8).add(size) as usize) == (curr as usize) {
                (*block).size += (*curr).size;
                (*block).next = (*curr).next;
            }
            return;
        }

        let mut prev = curr;
        let mut next = (*curr).next;
        while !next.is_null() && (next as usize) < (block as usize) {
            prev = next;
            next = (*next).next;
        }
        (*block).next = next;
        (*prev).next = block;

        // Coalesce with next
        if !next.is_null() && ((block as *mut u8).add(size) as usize) == (next as usize) {
            (*block).size += (*next).size;
            (*block).next = (*next).next;
        }
        // Coalesce with prev
        if ((prev as *mut u8).add((*prev).size) as usize) == (block as usize) {
            (*prev).size += (*block).size;
            (*prev).next = (*block).next;
        }
    }
}

#[global_allocator]
static ALLOCATOR: Xv8Alloc = Xv8Alloc {
    free_list: core::sync::atomic::AtomicPtr::new(ptr::null_mut()),
};

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
    let _ = argc;
    let _ = argv;
    lang_start(safe_main, argc, argv, 0);
    xv8_libc::exit(0)
}
