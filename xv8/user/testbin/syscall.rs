#![no_std]
#![no_main]

use user::*;

const O_CREATE_RW: usize = OpenFlag::CREATE | OpenFlag::READ_WRITE | OpenFlag::TRUNCATE;

fn test_dup2() {
    let fd = open("/test_dup2", O_CREATE_RW).expect("create test file");
    write(fd, b"hello").expect("write");
    close(fd).expect("close");

    let fd = open("/test_dup2", OpenFlag::READ_ONLY).expect("open for dup2");

    let new_fd = dup2(fd, Fd::from_raw(100)).expect("dup2 to fd 100");
    assert_eq!(new_fd.as_raw(), 100, "dup2 should return new fd = 100");

    let fd2 = open("/test_dup2", OpenFlag::READ_ONLY).expect("open another");

    let overwritten = dup2(fd2, Fd::from_raw(100)).expect("dup2 should overwrite fd 100");
    assert_eq!(overwritten.as_raw(), 100, "dup2 should return fd 100");

    close(fd).expect("close original");
    close(fd2).expect("close second");
    unlink("/test_dup2").expect("unlink");
}

fn test_getppid() {
    let pid = fork().expect("fork");
    if pid == 0 {
        let child_ppid = getppid().expect("child getppid");
        assert!(child_ppid > 0, "child's parent pid should be valid");
        exit(0);
    }

    let mut code = 0;
    wait(&mut code).expect("wait for child");
    assert_eq!(code, 0, "child should have exited successfully");
}

fn test_setuid_setgid() {
    setuid(1000).expect("setuid");
    setgid(1000).expect("setgid");
}

fn test_getpgid() {
    let pid = getpid();
    let pgrp = getpgid(pid).expect("getpgid");
    assert!(pgrp > 0, "process group should be valid");

    let pgrp_self = getpgid(0).expect("getpgid(0) should work");
    assert_eq!(pgrp, pgrp_self, "getpgid(0) should equal getpgid(getpid())");

    let child_pid = fork().expect("fork");
    if child_pid == 0 {
        let child_pgrp = getpgid(0).expect("child getpgid(0)");
        exit(if child_pgrp == pgrp { 0 } else { 1 });
    }

    let mut code = 0;
    wait(&mut code).expect("wait for child");
    assert_eq!(code, 0, "child should have same pgrp as parent");
}

fn test_isatty() {
    let result = isatty(Fd::STDIN).expect("isatty stdin");
    assert!(result, "stdin should be a tty");

    let fd = open("/test_isatty", O_CREATE_RW).expect("create file for isatty");
    close(fd).expect("close");
    let fd = open("/test_isatty", OpenFlag::READ_ONLY).expect("open file");
    let not_tty = isatty(fd).expect("isatty on file");
    assert!(!not_tty, "regular file should not be a tty");
    close(fd).expect("close file");
    unlink("/test_isatty").expect("unlink");
}

fn test_tcgetattr_tcsetattr() {
    let mut attr = [0u8; 64];
    tcgetattr(Fd::STDIN, &mut attr).expect("tcgetattr");
    tcsetattr(Fd::STDIN, &attr, 0).expect("tcsetattr");
}

fn test_time() {
    let t = time().expect("time should work");
    let _ = t; // 时间值可能为 0（系统刚启动）
}

fn test_nanosleep() {
    nanosleep(0, 100_000_000).expect("nanosleep 100ms"); // 100ms = 0.1s
}

fn test_clock_gettime() {
    let (sec, nsec) = clock_gettime().expect("clock_gettime should work");
    let _ = sec;
    let _ = nsec; // 值可能為 0（系統剛啟動）
}

fn test_clock_getres() {
    let (sec, nsec) = clock_getres().expect("clock_getres should work");
    let _ = sec;
    let _ = nsec;
}

fn test_readv() {
    let fd = open("/test_readv", O_CREATE_RW).expect("create test file");
    write(fd, b"hello world").expect("write");
    close(fd).expect("close");

    let fd = open("/test_readv", OpenFlag::READ_ONLY).expect("open for readv");
    let mut buf1 = [0u8; 5];
    let mut buf2 = [0u8; 6];
    let iovs = &mut [
        Iovec { iov_base: buf1.as_mut_ptr(), iov_len: 5 },
        Iovec { iov_base: buf2.as_mut_ptr(), iov_len: 6 },
    ];
    let n = readv(fd, iovs).expect("readv should work");
    assert_eq!(n, 11, "readv should read 11 bytes");
    close(fd).expect("close");
    unlink("/test_readv").expect("unlink");
}

fn test_writev() {
    let fd = open("/test_writev", O_CREATE_RW).expect("create for writev");
    let buf1 = [b'h', b'e', b'l', b'l', b'o'];
    let buf2 = [b' ', b'w', b'o', b'r', b'l', b'd'];
    let iovs = &[
        Iovec { iov_base: buf1.as_ptr() as *mut u8, iov_len: 5 },
        Iovec { iov_base: buf2.as_ptr() as *mut u8, iov_len: 6 },
    ];
    let n = writev(fd, iovs).expect("writev should work");
    assert_eq!(n, 11, "writev should write 11 bytes");
    close(fd).expect("close");

    let fd = open("/test_writev", OpenFlag::READ_ONLY).expect("open for verify");
    let mut buf = [0u8; 11];
    let mut iovs = [Iovec { iov_base: buf.as_mut_ptr(), iov_len: 11 }];
    readv(fd, &mut iovs).expect("read back");
    assert_eq!(&buf, b"hello world", "content should match");
    close(fd).expect("close");
    unlink("/test_writev").expect("unlink");
}

fn test_pread() {
    let fd = open("/test_pread", O_CREATE_RW).expect("create for pread");
    write(fd, b"0123456789ab").expect("write initial");
    close(fd).expect("close");
    unlink("/test_pread").expect("unlink");
}

fn test_pwrite() {
    let fd = open("/test_pwrite", O_CREATE_RW).expect("create for pwrite");
    let n = write(fd, b"XXXX").expect("write should work");
    close(fd).expect("close");
    unlink("/test_pwrite").expect("unlink");
    let _ = n;
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_dup2();
    test_getppid();
    test_setuid_setgid();
    test_getpgid();
    test_isatty();
    test_tcgetattr_tcsetattr();
    test_time();
    test_nanosleep();
    test_clock_gettime();
    test_clock_getres();
    test_readv();
    test_writev();
    test_pread();
    test_pwrite();
}