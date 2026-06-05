#![no_std]
#![no_main]

use user::*;

fn check(test: &str, ok: bool) {
    if ok { println!("  {} ... ok", test); }
    else { println!("  {} ... FAILED", test); exit(1); }
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("_httpepoll: async HTTP server + epoll...");

    // Test 1: epoll_create1
    let epfd = epoll_create1(0).expect("epoll_create1");
    check("epoll_create1", epfd.as_raw() > 2);

    // Test 2: epoll_wait with 0 timeout returns 0 immediately
    let mut events = [kernel::abi::EpollEvent { events: 0, data: 0 }; 4];
    let n = epoll_wait(epfd, &mut events, 0).expect("epoll_wait empty");
    check("epoll_wait empty timeout=0", n == 0);

    // Test 3: fork server + client with epoll-based async
    let port = 27001u16;
    let srv = tcp_socket().expect("srv socket");
    tcp_bind(srv, port).expect("bind");
    tcp_listen(srv).expect("listen");
    check("tcp_listen", true);

    // Register listen socket with epoll
    let ev = kernel::abi::EpollEvent {
        events: kernel::abi::EPOLLIN,
        data: srv.as_raw() as u64,
    };
    epoll_ctl(epfd, kernel::abi::EPOLL_CTL_ADD, srv, Some(&ev)).expect("epoll_ctl add srv");

    // Fork: child runs epoll accept loop, parent connects and sends
    println!("  fork ...");
    match fork().expect("fork") {
        0 => {
            // Child: epoll-based accept loop
            let mut events = [kernel::abi::EpollEvent { events: 0, data: 0 }; 16];
            let mut served = 0u32;
            loop {
                println!("  child: epoll_wait...");
                let n = epoll_wait(epfd, &mut events, -1).expect("epoll_wait");
                println!("  child: epoll_wait returned n={}", n);
                for i in 0..n {
                    let fd = events[i].data as usize;
                    println!("  child: event[{}].fd={}, srv={}", i, fd, srv.as_raw());
                    if fd == srv.as_raw() {
                        // Accept one connection per event (level-triggered epoll
                        // will re-arm if more are pending)
                        match tcp_accept(srv) {
                            Ok(cli) => {
                                println!("  child: accepted cli={}", cli.as_raw());
                                let ev2 = kernel::abi::EpollEvent {
                                    events: kernel::abi::EPOLLIN,
                                    data: cli.as_raw() as u64,
                                };
                                let _ = epoll_ctl(epfd, kernel::abi::EPOLL_CTL_ADD, cli, Some(&ev2));
                            }
                            Err(_) => {}
                        }
                    } else {
                        let cli = Fd::from_raw(fd);
                        let mut buf = [0u8; 1024];
                        match tcp_recv(cli, &mut buf) {
                            Ok(n) if n > 0 => {
                                let resp = b"ok\n";
                                let _ = tcp_send(cli, resp);
                                let _ = epoll_ctl(epfd, kernel::abi::EPOLL_CTL_DEL, cli, None);
                                close(cli).expect("close cli");
                                served += 1;
                                if served >= 2 {
                                    close(srv).expect("close srv");
                                    exit(0);
                                }
                            }
                            _ => {
                                let _ = epoll_ctl(epfd, kernel::abi::EPOLL_CTL_DEL, cli, None);
                                close(cli).expect("close cli");
                            }
                        }
                    }
                }
            }
        }
        _parent_pid => {
            println!("  parent: nanosleep + connect");
            let _ = nanosleep(0, 200_000_000);
            // Parent: connect and send 2 requests
            for i in 0..2 {
                let cli = tcp_socket().expect("cli socket");
                tcp_connect(cli, &kernel::abi::Ipv4Addr::LOOPBACK.0, port)
                    .expect("connect");
                let _ = tcp_send(cli, b"GET / HTTP/1.0\r\n\r\n");
                let mut buf = [0u8; 64];
                let n = tcp_recv(cli, &mut buf).expect("recv");
                if n > 0 && buf[..n].starts_with(b"ok") {
                    println!("  request {} ... ok", i + 1);
                } else {
                    println!("  request {} ... FAILED", i + 1);
                    exit(1);
                }
                close(cli).expect("close cli");
            }
            let mut status = 0;
            wait(&mut status).expect("wait");
            if status == 0 {
                println!("_httpepoll: PASS");
                exit(0);
            } else {
                println!("_httpepoll: FAILED (server exit={})", status);
                exit(1);
            }
        }
    }
}
