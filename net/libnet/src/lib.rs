#![cfg_attr(feature = "xv8", no_std)]
#[cfg(feature = "xv8")]
extern crate alloc;

pub mod dns;
#[cfg(not(feature = "xv8"))]
pub mod icmp;
pub mod ntp;
pub mod tftp;
pub mod util;
pub mod net_impl;

use core::sync::atomic::AtomicU16;
use core::sync::atomic::Ordering;
static DNS_ID: AtomicU16 = AtomicU16::new(1);

/// Generate a pseudo-random u16 for DNS query IDs.
/// Uses an atomic counter — sufficient for DNS transaction IDs.
pub fn random_u16() -> u16 {
    DNS_ID.fetch_add(1, Ordering::Relaxed)
}
