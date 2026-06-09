#![no_std]
#![no_main]

use user::*;

fn uint_to_str(mut n: usize, buf: &mut [u8]) -> usize {
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    buf.len() - i
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("container test: stage 1 - setup");

    let _ = mkdir("/mycontainer");
    let _ = mkdir("/mycontainer/oldroot");
    let _ = mkdir("/mycontainer/dev");
    let _ = mknod("/mycontainer/dev/console", 1, 3);

    let cg = match open("cgroup", OpenFlag::READ_WRITE) {
        Ok(fd) => fd,
        Err(_) => exit_with_msg("open /cgroup failed"),
    };
    let cmd = b"create testcontainer\n";
    if write(cg, cmd).is_err() {
        exit_with_msg("create cgroup failed");
    }

    println!("container test: stage 2 - fork + full isolation");

    match fork() {
        Ok(0) => {
            if unshare(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS).is_err() {
                exit_with_msg("unshare failed");
            }

            let _ = sethostname(b"testcontainer");

            let pid = getpid();
            let mut num = [0u8; 16];
            let plen = uint_to_str(pid, &mut num);
            let attach_prefix = b"attach ";
            let attach_suffix = b" testcontainer\n";
            let mut acmd = [0u8; 64];
            acmd[..attach_prefix.len()].copy_from_slice(attach_prefix);
            let start = attach_prefix.len();
            acmd[start..start + plen].copy_from_slice(&num[num.len() - plen..]);
            acmd[start + plen..start + plen + attach_suffix.len()].copy_from_slice(attach_suffix);
            let acmd_len = attach_prefix.len() + plen + attach_suffix.len();
            if write(cg, &acmd[..acmd_len]).is_err() {
                exit_with_msg("cgroup attach failed");
            }
            let _ = close(cg);

            match pivot_root("/mycontainer", "/mycontainer/oldroot") {
                Ok(()) => {}
                Err(e) => {
                    println!("pivot_root failed errno={}", e.as_code());
                    exit(1);
                }
            }
            let _ = chdir("/");

            match open("/mycontainer", OpenFlag::READ_ONLY) {
                Ok(fd) => {
                    close(fd).unwrap();
                    println!("FAIL: /mycontainer still accessible after pivot_root");
                    exit(1);
                }
                Err(_) => {}
            }

            match open("/oldroot", OpenFlag::READ_ONLY) {
                Ok(fd) => {
                    close(fd).unwrap();
                }
                Err(_) => {
                    println!("FAIL: /oldroot not accessible after pivot_root");
                    exit(1);
                }
            }

            match open("/dev/console", OpenFlag::READ_ONLY) {
                Ok(fd) => {
                    close(fd).unwrap();
                }
                Err(_) => {
                    println!("FAIL: /dev/console not accessible");
                    exit(1);
                }
            }

            exit(0);
        }
        Ok(pid) => {
            let _ = close(cg);

            let mut code = 0;
            match wait(&mut code) {
                Ok(wpid) if wpid == pid => {}
                _ => {
                    println!("wait failed");
                    exit(1);
                }
            }
            if code != 0 {
                println!("container child failed with code {}", code);
                exit(1);
            }

            let cg2 = match open("cgroup", OpenFlag::READ_WRITE) {
                Ok(fd) => fd,
                Err(_) => exit_with_msg("open /cgroup failed"),
            };
            let mut buf = [0u8; 512];
            let n = match read(cg2, &mut buf) {
                Ok(n) => n,
                Err(_) => exit_with_msg("read stats failed"),
            };
            let stats = core::str::from_utf8(&buf[..n]).unwrap_or("");
            if !stats.contains("testcontainer") {
                println!("FAIL: cgroup stats missing testcontainer: {}", stats);
                exit(1);
            }
            let _ = close(cg2);

            println!("container test passed");
        }
        Err(_) => {
            println!("fork failed");
            exit(1);
        }
    }
}
