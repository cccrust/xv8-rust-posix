#![no_std]
#![no_main]

use user::*;

const MAX_RETRIES: usize = 100;

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
fn main(args: Args) {
    if args.len() < 2 {
        eprintln!("usage: listen <port>");
        exit(1);
    }

    let port: u16 = match args.get_str(1).unwrap().parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("invalid port");
            exit(1);
        }
    };

    let fd = socket(0).unwrap_or_else(|_| {
        eprintln!("socket failed");
        exit(1);
    });

    wait_for_dhcp(fd);

    let bind_fd = socket(port).unwrap_or_else(|_| {
        eprintln!("bind to port {} failed", port);
        exit(1);
    });

    println!("listening on UDP port {}", port);

    loop {
        let mut buf = [0u8; 512];
        let mut src_ip = [0u8; 4];
        let mut src_port = 0u16;

        match receive(bind_fd, &mut buf, &mut src_ip, &mut src_port) {
            Ok(n) => {
                let from = Ipv4Addr(src_ip);
                let msg = core::str::from_utf8(&buf[..n]).unwrap_or("<binary>");
                println!("{}:{} - {}", from, src_port, msg);
            }
            Err(_) => {
                let _ = sleep(1);
            }
        }
    }
}
