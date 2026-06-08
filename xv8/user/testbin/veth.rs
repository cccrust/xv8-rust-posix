#![no_std]
#![no_main]

use user::*;

/// Ioctl arg structure: two 16-byte interface names
#[repr(C)]
struct VethReq {
    name1: [u8; 16],
    name2: [u8; 16],
}

fn make_req(n1: &str, n2: &str) -> VethReq {
    let mut req = VethReq { name1: [0u8; 16], name2: [0u8; 16] };
    let b1 = n1.as_bytes();
    let b2 = n2.as_bytes();
    let n1c = b1.len().min(15);
    let n2c = b2.len().min(15);
    req.name1[..n1c].copy_from_slice(&b1[..n1c]);
    req.name2[..n2c].copy_from_slice(&b2[..n2c]);
    req
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    // Open a UDP socket to get an fd for ioctl
    let fd = match socket(0) {
        Ok(fd) => fd,
        Err(_) => exit_with_msg("socket failed"),
    };

    // Create veth pair
    let req = make_req("veth-host", "veth-ctr");
    match ioctl(fd, 100, &req as *const _ as usize) {
        Ok(0) => {}
        Ok(_) => exit_with_msg("veth: unexpected return"),
        Err(_e) => exit_with_msg("ioctl failed"),
    }

    let _ = close(fd);

    println!("veth test passed");
    exit(0);
}
