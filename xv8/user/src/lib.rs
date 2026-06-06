#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

#[macro_use]
mod io;
mod args;
mod line;
mod syscall;

pub use kernel::abi::*;

pub use args::*;
pub use io::{Read, Stderr, Stdin, Stdout, Write};
pub use line::LineEditor;
pub use syscall::*;

/// Simple bump allocator for user-space programs using `sbrk`.
/// Memory is never freed individually; it's reclaimed when the process exits.
struct SbrkAlloc;

unsafe impl GlobalAlloc for SbrkAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = if layout.size() < 1 { 1 } else { layout.size() };
        let old_brk = crate::syscall::raw::sbrk(size);
        if old_brk <= 0 {
            core::ptr::null_mut()
        } else {
            old_brk as *mut u8
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static SBRK_ALLOC: SbrkAlloc = SbrkAlloc;

unsafe extern "Rust" {
    /// The entry point for user programs.
    /// This function is called by the user entry `_start()` with the command-line arguments passed in as `args`.
    /// User binaries must define this function with the same signature and with `no_mangle` attribute.
    fn main(args: Args);
}

/// The entry point for user programs.
/// This function is mapped to the `.text.entry` section, which is the entry point for user processes.
///
/// Whichever elf binary is currently loaded in the memory, this function will jump to that binary's `main()` function.
/// Before the jump happens, the command-line arguments are extracted from the stack and passed to `main()`.
/// After `main()` returns, the process exits with a status code of 0.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
fn _start() -> ! {
    unsafe {
        let args = Args::from_stack();
        main(args);
        exit(0);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("! {}", info);
    exit(1)
}
