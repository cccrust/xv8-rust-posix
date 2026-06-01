#![no_std]
#![no_main]

use user::*;

const PING_DATA: &[u8] = b"netping test";
const GATEWAY: [u8; 4] = [10, 0, 2, 2];
const MAX_RETRIES: usize = 100;
const TIMEOUT_TICKS: usize = 100;

fn wait_for_dhcp(fd: Fd) {
    let gw = [10, 0, 2, 2];
    let payload = b"dhcp probe";
    for _ in 0..MAX_RETRIES {
        match send(fd, payload, &gw, 9999) {
            Ok(_) => return,
            Err(e) => {
                assert_eq!(e, SysError::NoEntry);
                let _ = sleep(5);
            }
        }
    }
    panic!("DHCP did not complete");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let udp_fd = socket(0).expect("socket open");
    wait_for_dhcp(udp_fd);
    close(udp_fd).expect("close udp");

    let fd = pingsocket().expect("pingsocket open");

    let mut received = false;
    for attempt in 0..3 {
        pingsend(fd, PING_DATA, &GATEWAY).expect("pingsend");

        let mut buf = [0u8; 64];
        let mut src_ip = [0u8; 4];

        let mut waited = 0;
        while waited < TIMEOUT_TICKS {
            if let Ok(_n) = pingrecv(fd, &mut buf, &mut src_ip) {
                assert_eq!(src_ip, GATEWAY, "reply must come from gateway");
                received = true;
                break;
            }
            let _ = sleep(1);
            waited += 1;
        }

        if received {
            break;
        }

        if attempt + 1 < 3 {
            let _ = sleep(10);
        }
    }

    assert!(received, "ping to gateway timed out after 3 attempts");

    close(fd).expect("close ping");
}
