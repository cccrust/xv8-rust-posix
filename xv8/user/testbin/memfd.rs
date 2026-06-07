#![no_std]
#![no_main]

use user::*;

fn test_memfd_create_write_read() {
    let fd = raw::memfd_create(0);
    assert!(fd >= 0, "memfd_create failed: {}", fd);
    let fd = Fd::from_raw(fd as usize);

    let data = b"hello memfd";
    let n = write(fd, data).expect("write to memfd");
    assert_eq!(n, data.len(), "write returned {n}");

    lseek(fd, 0, 0).expect("lseek to start");
    let mut buf = [0u8; 64];
    let n = read(fd, &mut buf).expect("read from memfd");
    assert_eq!(n, data.len(), "read returned {n}");
    assert_eq!(&buf[..n], data, "read data mismatch");

    close(fd).expect("close");
    println!("ok memfd_create_write_read");
}

fn test_memfd_seek_extend() {
    let fd = raw::memfd_create(0);
    assert!(fd >= 0, "memfd_create failed: {}", fd);
    let fd = Fd::from_raw(fd as usize);

    let data = b"0123456789";
    write(fd, data).expect("write initial data");

    lseek(fd, 100, 0).expect("lseek to 100");
    write(fd, b"end").expect("write at offset 100");

    let size = lseek(fd, 0, 1).expect("lseek to get size");
    assert_eq!(size, 103, "file size should be 103");

    lseek(fd, 0, 0).expect("lseek to start");
    let mut buf = [0u8; 200];
    let n = read(fd, &mut buf).expect("read all");
    assert_eq!(&buf[..10], data, "start data mismatch");
    assert_eq!(&buf[100..103], b"end", "end data mismatch");

    close(fd).expect("close");
    println!("ok memfd_seek_extend");
}

fn test_memfd_cloexec() {
    let fd = raw::memfd_create(user::MFD_CLOEXEC as usize);
    assert!(fd >= 0, "memfd_create(MFD_CLOEXEC) failed: {}", fd);
    let fd = Fd::from_raw(fd as usize);

    write(fd, b"test").expect("write");
    lseek(fd, 0, 0).expect("lseek");
    let mut buf = [0u8; 8];
    read(fd, &mut buf).expect("read");
    assert_eq!(&buf[..4], b"test", "data mismatch with cloexec");

    close(fd).expect("close");
    println!("ok memfd_cloexec");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_memfd_create_write_read();
    test_memfd_seek_extend();
    test_memfd_cloexec();
}
