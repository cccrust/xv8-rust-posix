#![no_std]
#![cfg_attr(feature = "standalone", feature(lang_items))]
#![cfg_attr(feature = "standalone", allow(internal_features))]

extern crate alloc;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::io::_print(format_args!("\n"))
    };
    ($($arg:tt)*) => {{
        $crate::io::_print(format_args!($($arg)*));
        $crate::io::_print(format_args!("\n"));
    }};
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        $crate::io::_eprint(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::io::_eprint(format_args!("\n"))
    };
    ($($arg:tt)*) => {{
        $crate::io::_eprint(format_args!($($arg)*));
        $crate::io::_eprint(format_args!("\n"));
    }};
}

#[cfg(feature = "standalone")]
mod runtime;

pub mod io;
pub mod fmt {
pub use core::fmt::{
    self, Arguments, Debug, Display, Formatter, Result, Write,
    Error,
};
    pub fn format(args: core::fmt::Arguments<'_>) -> alloc::string::String {
        alloc::fmt::format(args)
    }
}

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        $crate::fmt::format(format_args!($($arg)*))
    }};
}

pub use alloc::boxed::Box;
pub mod path;
pub mod result {
    pub use core::result::*;
}
pub mod env;
pub mod fs;
pub mod process;
pub mod time;
pub mod thread;
pub mod ffi;
pub mod net;
pub mod mpsc;

pub use alloc::vec::Vec;
pub use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

#[macro_export]
macro_rules! vec {
    ($elem:expr; $n:expr) => {{
        let mut v = $crate::Vec::with_capacity($n);
        let e = $elem;
        for _ in 0..$n {
            v.push(e.clone());
        }
        v
    }};
    ($($x:expr),* $(,)?) => {{
        let mut v = $crate::Vec::new();
        $(v.push($x);)*
        v
    }};
}

pub mod cmp {
    pub use core::cmp::*;
}

pub mod mem {
    pub use core::mem::*;
}

pub mod slice {
    pub use core::slice::*;
}

pub mod ptr {
    pub use core::ptr::*;
}

pub mod iter {
    pub use core::iter::*;
}

pub mod char {
    pub use core::char::*;
}

pub mod error {
    pub use core::error::*;
}

pub mod str {
    pub use core::str::*;
}

pub mod ops {
    pub use core::ops::*;
}

pub mod hash {
    pub use core::hash::*;
}

pub mod any {
    pub use core::any::*;
}

pub mod convert {
    pub use core::convert::*;
}

pub mod sync;
pub mod panic;

pub mod prelude {
    pub mod rust_2021 {
        pub use core::prelude::rust_2021::*;
        pub use alloc::boxed::Box;
        pub use alloc::string::{String, ToString};
        pub use alloc::vec::Vec;
        pub use crate::vec;
        pub use crate::format;
        pub use crate::{println, print, eprintln, eprint};
    }
    pub mod v1 {
        pub use core::prelude::v1::*;
        pub use alloc::boxed::Box;
        pub use alloc::string::{String, ToString};
        pub use alloc::vec::Vec;
        pub use crate::vec;
        pub use crate::format;
        pub use crate::{println, print, eprintln, eprint};
    }
}

pub mod collections {
    extern crate alloc;
    pub use alloc::collections::*;
    pub use hashbrown::HashMap;
    pub use hashbrown::HashSet;
}

pub mod os {
    pub mod unix {
        pub mod fs;
        pub mod io;
        pub mod net;
        pub mod process;
    }
    pub mod linux {
        pub mod eventfd;
        pub mod timerfd;
        pub mod memfd;
        pub mod pidfd;
    }
    pub mod fd {
        pub use super::unix::io::{
            AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd,
        };
    }
}
