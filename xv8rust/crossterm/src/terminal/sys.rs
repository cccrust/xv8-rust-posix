//! This module provides platform related functions.

#[cfg(all(unix, not(target_arch = "riscv64")))]
#[cfg(feature = "events")]
pub use self::unix::supports_keyboard_enhancement;
#[cfg(all(unix, not(target_arch = "riscv64")))]
pub(crate) use self::unix::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, size, window_size,
};
#[cfg(target_arch = "riscv64")]
pub(crate) use self::riscv64::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, size, window_size,
};
#[cfg(target_arch = "riscv64")]
#[cfg(feature = "events")]
pub use self::riscv64::supports_keyboard_enhancement;
#[cfg(windows)]
#[cfg(feature = "events")]
pub use self::windows::supports_keyboard_enhancement;
#[cfg(all(windows, test))]
pub(crate) use self::windows::temp_screen_buffer;
#[cfg(windows)]
pub(crate) use self::windows::{
    clear, disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, scroll_down, scroll_up,
    set_size, set_window_title, size, window_size,
};

#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub mod file_descriptor;
#[cfg(all(unix, not(target_arch = "riscv64")))]
mod unix;
#[cfg(target_arch = "riscv64")]
mod riscv64;
