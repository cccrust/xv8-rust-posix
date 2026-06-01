#![no_std]
#![no_main]

use xv8_libc::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    
    macro_rules! test {
        ($name:expr, $body:expr) => {{
            let ret = unsafe { $body };
            if ret >= 0 {
                println!("  PASS: {}", $name);
                passed += 1;
            } else {
                eprintln!("  FAIL: {} (errno={})", $name, -ret);
                failed += 1;
            }
        }};
    }
    
    macro_rules! test_expr {
        ($name:expr, $cond:expr) => {{
            if $cond {
                println!("  PASS: {}", $name);
                passed += 1;
            } else {
                eprintln!("  FAIL: {}", $name);
                failed += 1;
            }
        }};
    }
    
    println!("syscall: mkdir /testdir");
    test!("mkdir", mkdir("/testdir\0".as_ptr(), 0o755));
    
    // Wait for disk
    for _ in 0..100 { unsafe { core::hint::spin_loop(); } }
    
    test!("mkdir again (EEXIST)", mkdir("/testdir\0".as_ptr(), 0o755));
    
    test!("unlink testdir", unlink("/testdir\0".as_ptr()));
    
    test!("mkdir /subdir", mkdir("/subdir\0".as_ptr(), 0o755));
    test!("rmdir /subdir", unlink("/subdir\0".as_ptr()));
    
    // Create a file via open+write+close
    let fd = open("/testfile\0".as_ptr(), (OpenFlag::CREATE | OpenFlag::WRITE_ONLY) as usize);
    test_expr!("open /testfile for write", fd >= 0);
    if fd >= 0 {
        let data = b"hello xv8 test\n";
        test!("write", write(fd as usize, data.as_ptr(), data.len()));
        test!("close", close(fd as usize));
    }
    
    // Stat the file
    let mut st = Stat::default();
    let sfd = open("/testfile\0".as_ptr(), OpenFlag::READ_ONLY as usize);
    if sfd >= 0 {
        test!("fstat", fstat(sfd as usize, &mut st as *mut _));
        test_expr!("file size", st.size == 15);
        test_expr!("file mode != 0", st.mode != 0);
        test!("close sfd", close(sfd as usize));
    }
    
    // Read the file back
    let rfd = open("/testfile\0".as_ptr(), OpenFlag::READ_ONLY as usize);
    if rfd >= 0 {
        let mut buf = [0u8; 64];
        let n = read(rfd as usize, buf.as_mut_ptr(), buf.len());
        test_expr!("read", n >= 0);
        if n > 0 {
            let content = core::str::from_utf8(&buf[..n as usize]).unwrap_or("");
            test_expr!("file content", content.contains("hello xv8 test"));
        }
        test!("close rfd", close(rfd as usize));
    }
    
    // Symlink check
    test!("rename", rename("/testfile\0".as_ptr(), "/renamed\0".as_ptr()));
    
    // Access check
    test!("access F_OK", access("/renamed\0".as_ptr(), 0));
    test!("access R_OK", access("/renamed\0".as_ptr(), 4));
    
    // Unlink
    test!("unlink /renamed", unlink("/renamed\0".as_ptr()));
    
    // Symlink
    let fd2 = open("/symtarget\0".as_ptr(), (OpenFlag::CREATE | OpenFlag::WRITE_ONLY) as usize);
    if fd2 >= 0 {
        let _ = write(fd2 as usize, b"symdata\n".as_ptr(), 8);
        let _ = close(fd2 as usize);
    }
    test!("symlink", symlink("/symtarget\0".as_ptr(), "/symlink\0".as_ptr()));
    
    let mut linkbuf = [0u8; 64];
    let n = readlink("/symlink\0".as_ptr(), linkbuf.as_mut_ptr(), linkbuf.len());
    test_expr!("readlink", n >= 0);
    if n > 0 {
        let target = core::str::from_utf8(&linkbuf[..n as usize]).unwrap_or("");
        test_expr!("symlink target", target.contains("symtarget"));
    }
    
    // Hard link
    test!("link", link("/symtarget\0".as_ptr(), "/hardlink\0".as_ptr()));
    
    // Chmod
    test!("chmod", chmod("/symtarget\0".as_ptr(), 0o600));
    
    // Get uid/gid
    let uid = getuid();
    test_expr!("getuid", uid >= 0);
    let gid = getgid();
    test_expr!("getgid", gid >= 0);
    
    // Unlink remaining
    let _ = unlink("/symtarget\0".as_ptr());
    let _ = unlink("/symlink\0".as_ptr());
    let _ = unlink("/hardlink\0".as_ptr());
    
    // Truncate
    let fd3 = open("/truncfile\0".as_ptr(), (OpenFlag::CREATE | OpenFlag::WRITE_ONLY) as usize);
    if fd3 >= 0 {
        let _ = write(fd3 as usize, b"1234567890\n".as_ptr(), 11);
        let _ = close(fd3 as usize);
    }
    test!("truncate", truncate("/truncfile\0".as_ptr(), 5));
    let _ = unlink("/truncfile\0".as_ptr());
    
    println!("\n=== Results ===");
    println!("PASS: {}/{}", passed, passed + failed);
    println!("FAIL: {}/{}", failed, passed + failed);
    
    if failed > 0 {
        exit(1);
    } else {
        exit(0);
    }
}

// Minimal Args struct to match xv8-libc's expectation
use core::fmt::Write;
struct Args {
    argc: usize,
    argv: *const *const u8,
}

impl Args {
    fn get(&self, i: usize) -> &str {
        if i >= self.argc { return ""; }
        unsafe {
            let ptr = self.argv.add(i).read();
            let mut len = 0;
            while *ptr.add(len) != 0 { len += 1; }
            core::str::from_utf8(core::slice::from_raw_parts(ptr, len)).unwrap_or("")
        }
    }
}
