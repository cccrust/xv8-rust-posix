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
    let fd = match open("cgroup", OpenFlag::READ_WRITE) {
        Ok(fd) => fd,
        Err(_) => exit_with_msg("open /cgroup failed"),
    };

    // Create a cgroup
    let cmd = b"create testgroup\n";
    if write(fd, cmd).is_err() {
        exit_with_msg("create cgroup failed");
    }

    // Set cpu.max
    let cmd = b"cpu.max 50000 100000 testgroup\n";
    if write(fd, cmd).is_err() {
        exit_with_msg("cpu.max failed");
    }

    // Set memory.max
    let cmd = b"memory.max 1048576 testgroup\n";
    if write(fd, cmd).is_err() {
        exit_with_msg("memory.max failed");
    }

    // Set pids.max
    let cmd = b"pids.max 10 testgroup\n";
    if write(fd, cmd).is_err() {
        exit_with_msg("pids.max failed");
    }

    // Build "attach <pid> testgroup\n" command manually
    let pid = getpid();
    let mut num = [0u8; 16];
    let plen = uint_to_str(pid, &mut num);
    let attach_prefix = b"attach ";
    let attach_suffix = b" testgroup\n";
    let mut cmd = [0u8; 64];
    cmd[..attach_prefix.len()].copy_from_slice(attach_prefix);
    let digit_src = &num[num.len() - plen..];
    let start = attach_prefix.len();
    cmd[start..start + plen].copy_from_slice(digit_src);
    cmd[start + plen..start + plen + attach_suffix.len()].copy_from_slice(attach_suffix);
    let cmd_len = attach_prefix.len() + plen + attach_suffix.len();
    if write(fd, &cmd[..cmd_len]).is_err() {
        exit_with_msg("attach failed");
    }

    // Read stats and verify
    let mut buf = [0u8; 512];
    let n = match read(fd, &mut buf) {
        Ok(n) => n,
        Err(_) => exit_with_msg("read stats failed"),
    };

    let stats = core::str::from_utf8(&buf[..n]).unwrap_or("");
    if !stats.contains("testgroup") {
        exit_with_msg("stats missing testgroup");
    }
    if !stats.contains("cpu: max=50000 period=100000") {
        exit_with_msg("stats missing cpu.max");
    }
    if !stats.contains("memory: max=1048576") {
        exit_with_msg("stats missing memory.max");
    }
    if !stats.contains("pids: max=10") {
        exit_with_msg("stats missing pids.max");
    }

    let _ = close(fd);
    exit(0);
}
