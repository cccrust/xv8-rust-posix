#![no_std]
#![no_main]

use user::*;

/// Match kernel's CapUserData layout: 3 x u32 (12 bytes total)
#[repr(C)]
struct CapData {
    effective: u32,
    permitted: u32,
    inheritable: u32,
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    // Test: read current capabilities
    let mut data = CapData { effective: 0, permitted: 0, inheritable: 0 };
    let ret = unsafe { raw::capget(core::ptr::null(), &mut data as *mut _ as *mut usize) };
    if ret < 0 {
        exit_with_msg("capget failed");
    }
    // Default should have all cap bits set (u32 max)
    if data.effective != !0u32 || data.permitted != !0u32 || data.inheritable != !0u32 {
        exit_with_msg("unexpected default caps");
    }

    // Test: set capabilities (drop CAP_NET_RAW = bit 13)
    let expected = !0u32 & !(1 << 13);
    let new = CapData {
        effective: expected,
        permitted: expected,
        inheritable: !0u32,
    };
    let ret = unsafe { raw::capset(core::ptr::null(), &new as *const _ as *const usize) };
    if ret < 0 {
        exit_with_msg("capset failed");
    }

    // Verify caps were dropped
    let mut verify = CapData { effective: 0, permitted: 0, inheritable: 0 };
    let ret = unsafe { raw::capget(core::ptr::null(), &mut verify as *mut _ as *mut usize) };
    if ret < 0 {
        exit_with_msg("capget after set failed");
    }
    if verify.effective != expected || verify.permitted != expected {
        exit_with_msg("caps mismatch after set");
    }

    exit(0);
}
