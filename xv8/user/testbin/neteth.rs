#![no_std]
#![no_main]

use user::*;

const GW: [u8; 4] = [10, 0, 2, 2];
const TEST_PORT: u16 = 9999;
const MAX_RETRIES: usize = 100;

/// Try sending a UDP packet to the QEMU gateway and wait for DHCP.
/// DHCP must complete before we can route to 10.0.2.0/24 (QEMU user-mode network).
fn wait_for_dhcp(fd: Fd) {
    let payload = b"dhcp probe";
    for _ in 0..MAX_RETRIES {
        match send(fd, payload, &GW, TEST_PORT) {
            Ok(n) => {
                assert_eq!(n, payload.len(), "send must return payload byte count");
                return;
            }
            Err(e) => {
                // No route yet; DHCP is still negotiating.
                assert_eq!(e, SysError::NoEntry, "expected NoEntry before DHCP");
                let _ = sleep(5);
            }
        }
    }
    panic!("DHCP did not complete within timeout");
}

fn test_udp_send_to_host() {
    let fd = socket(0).expect("socket open");

    // Wait for DHCP to assign an IP and add a route to 10.0.2.0/24.
    wait_for_dhcp(fd);

    // Now send a real packet. The send() syscall should succeed immediately.
    // (The packet will be queued until ARP resolves 10.0.2.2, then transmitted.
    //  QEMU user-mode gateway will silently drop it since nothing listens on :9999.)
    let payload = b"xv8 e1000 test";
    let n = send(fd, payload, &GW, TEST_PORT).expect("send to host");
    assert_eq!(n, payload.len(), "send must return payload byte count");

    close(fd).expect("close");
}

fn test_udp_listen_and_loopback_send() {
    // Open two sockets: one listens, one sends via loopback.
    let listen_fd = socket(26202).expect("listen socket open");
    let send_fd = socket(0).expect("send socket open");

    let payload = b"echo through e1000 interface";
    send(send_fd, payload, &Ipv4Addr::LOOPBACK.0, 26202).expect("send loopback");

    let mut buf = [0u8; 64];
    let mut src_ip = [0u8; 4];
    let mut src_port = 0u16;
    let n = receive(listen_fd, &mut buf, &mut src_ip, &mut src_port).expect("receive");

    assert_eq!(&buf[..n], payload, "loopback payload mismatch");
    assert_eq!(src_ip, Ipv4Addr::LOOPBACK.0, "source must be loopback");

    close(listen_fd).expect("close");
    close(send_fd).expect("close");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_udp_send_to_host();
    test_udp_listen_and_loopback_send();
}
