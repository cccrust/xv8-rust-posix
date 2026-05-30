#![cfg_attr(target_os = "none", no_std)]

pub mod io;
pub mod fmt;
pub mod opt;

pub use io::Read;
pub use io::Write;
