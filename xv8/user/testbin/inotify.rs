#![no_std]
#![no_main]

use user::*;

fn test_inotify_create() {
    let dir_fd = open("/", OpenFlag::READ_ONLY).expect("open root");
    close(dir_fd).expect("close root");

    let inot = raw::inotify_init1(0);
    assert!(inot >= 0, "inotify_init1 failed: {}", inot);
    let inot_fd = Fd::from_raw(inot as usize);

    let cpath = {
        let mut buf = [0u8; 128];
        buf[..1].copy_from_slice(b"/");
        buf
    };
    let wd = raw::inotify_add_watch(inot_fd.as_raw(), cpath.as_ptr(), kernel::abi::IN_CREATE) as i32;
    assert!(wd >= 0, "inotify_add_watch failed: {}", wd);
    assert_eq!(wd, 1, "first wd should be 1, got {}", wd);

    let fd = open("/x", OpenFlag::CREATE | OpenFlag::WRITE_ONLY)
        .expect("create /x");
    let msg = b"hello";
    write(fd, msg).expect("write");
    close(fd).expect("close");

    let mut buf = [0u8; 64];
    let n = read(inot_fd, &mut buf).expect("read inotify event");
    assert!(n >= 16, "read returned only {n} bytes");

    let event: kernel::abi::InotifyEvent = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const _) };
    assert_eq!(event.wd, wd, "wd mismatch: {} != {}", event.wd, wd);
    assert!(event.mask & kernel::abi::IN_CREATE != 0, "expected IN_CREATE, got {:x}", event.mask);
    assert!(event.len > 0, "expected non-zero name length");

    let name_end = 16 + event.len as usize;
    let child_name = core::str::from_utf8(&buf[16..name_end]).unwrap_or("");
    assert_eq!(child_name, "x", "child name: {child_name}");

    close(inot_fd).expect("close inotify fd");
    unlink("/x").expect("unlink");
    println!("ok inotify_create");
}

fn test_inotify_delete() {
    let inot = raw::inotify_init1(0);
    assert!(inot >= 0, "inotify_init1 failed: {}", inot);
    let inot_fd = Fd::from_raw(inot as usize);

    let cpath = {
        let mut buf = [0u8; 128];
        buf[..1].copy_from_slice(b"/");
        buf
    };
    let wd = raw::inotify_add_watch(inot_fd.as_raw(), cpath.as_ptr(), kernel::abi::IN_DELETE) as i32;
    assert!(wd >= 0, "inotify_add_watch failed: {}", wd);

    let fd = open("/y", OpenFlag::CREATE | OpenFlag::WRITE_ONLY)
        .expect("create /y");
    close(fd).expect("close");

    unlink("/y").expect("unlink");

    let mut buf = [0u8; 32];
    let n = read(inot_fd, &mut buf).expect("read inotify delete event");
    assert!(n >= 16, "read returned only {n} bytes");

    let event: kernel::abi::InotifyEvent = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const _) };
    assert_eq!(event.wd, wd, "wd mismatch");
    assert!(event.mask & kernel::abi::IN_DELETE != 0, "expected IN_DELETE, got {:x}", event.mask);

    close(inot_fd).expect("close inotify fd");
    println!("ok inotify_delete");
}

fn test_inotify_modify() {
    let fd = open("/z", OpenFlag::CREATE | OpenFlag::WRITE_ONLY)
        .expect("create /z");
    let data = b"some content";
    write(fd, data).expect("initial write");
    close(fd).expect("close initial");

    let inot = raw::inotify_init1(0);
    assert!(inot >= 0);
    let inot_fd = Fd::from_raw(inot as usize);

    let z_cpath = {
        let mut buf = [0u8; 128];
        buf[..2].copy_from_slice(b"/z");
        buf
    };
    let wd = raw::inotify_add_watch(inot_fd.as_raw(), z_cpath.as_ptr(), kernel::abi::IN_MODIFY) as i32;
    assert!(wd >= 0);

    let fd = open("/z", OpenFlag::WRITE_ONLY).expect("open for write");
    write(fd, b"m").expect("modify write");
    close(fd).expect("close");

    let mut buf = [0u8; 32];
    let n = read(inot_fd, &mut buf).expect("read inotify modify event");
    assert!(n >= 16);

    let event: kernel::abi::InotifyEvent = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const _) };
    assert_eq!(event.wd, wd, "wd mismatch");
    assert!(event.mask & kernel::abi::IN_MODIFY != 0, "expected IN_MODIFY, got {:x}", event.mask);

    close(inot_fd).expect("close inotify fd");
    unlink("/z").expect("unlink");
    println!("ok inotify_modify");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_inotify_create();
    test_inotify_delete();
    test_inotify_modify();
}
