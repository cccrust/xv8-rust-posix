#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};

use kernel::kalloc::KMEM;

/// Global allocator for the kernel binary.
/// Uses the kernel's buddy allocator (physical page allocator).
struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { KMEM.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { KMEM.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static KERNEL_ALLOC: KernelAlloc = KernelAlloc;

#[unsafe(export_name = "main")]
fn main() -> ! {
    kernel::main()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    kernel::panic_handler(info)
}
