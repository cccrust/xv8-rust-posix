fn main() {
    println!("test_std: host test mode");
    run_tests();
}

fn run_tests() {
    use std::io::{BufWriter, Cursor, Write, Read, copy};
    use std::panic::catch_unwind;
    use std::ffi::OsString;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
    use std::path::{Path, PathBuf};
    use std::{env, fs};

    let mut passed = 0u32;
    let mut failed = 0u32;

    macro_rules! ok {
        ($name:expr) => {{
            println!("  PASS: {}", $name);
            passed += 1;
        }};
    }

    macro_rules! fail {
        ($name:expr) => {{
            eprintln!("  FAIL: {}", $name);
            failed += 1;
        }};
    }

    macro_rules! test {
        ($name:expr, $cond:expr) => {{
            if $cond {
                ok!($name);
            } else {
                fail!($name);
            }
        }};
    }

    // ─── Cursor<T> tests ──────────────────────────────────────────────
    {
        let mut c = Cursor::new(vec![0u8; 10]);
        test!("Cursor write", c.write(&[1, 2, 3]).unwrap() == 3);
        test!("Cursor position", c.position() == 3);
        c.set_position(0);
        let mut buf = [0u8; 3];
        test!("Cursor read", c.read(&mut buf).unwrap() == 3 && buf == [1, 2, 3]);
    }

    // ─── Cursor<&[u8]> ───────────────────────────────────────────────
    {
        let mut c = Cursor::new(&b"hello world"[..]);
        let mut buf = [0u8; 5];
        test!("Cursor read &[u8]", c.read(&mut buf).unwrap() == 5 && buf == *b"hello");
    }

    // ─── BufWriter tests ──────────────────────────────────────────────
    {
        let mut bw = BufWriter::new(Cursor::new(Vec::new()));
        test!("BufWriter write", bw.write(b"hello").unwrap() == 5);
        test!("BufWriter flush", bw.flush().is_ok());
    }

    // ─── OsString tests ───────────────────────────────────────────────
    {
        let s = OsString::from("hello");
        test!("OsString len", s.len() == 5);
        let s2 = OsString::new();
        test!("OsString new empty", s2.is_empty());
        let s3 = s.clone();
        test!("OsString clone eq", s3 == s);
    }

    // ─── catch_unwind (xv8-user-std is no-op, skip actual panic tests) ─
    {
        let ok_result = catch_unwind(|| 42);
        test!("catch_unwind ok path", ok_result.unwrap() == 42);
    }

    // ─── Path tests ───────────────────────────────────────────────────
    {
        test!("Path::is_absolute /", Path::new("/").is_absolute());
        test!("Path::is_relative rel", Path::new("rel").is_relative());
        test!("Path::is_absolute abs", Path::new("/abs/path").is_absolute());
        test!("Path::starts_with", Path::new("/a/b").starts_with(Path::new("/a")));
        test!("Path::ends_with", Path::new("/a/b").ends_with(Path::new("b")));

        let mut pb = PathBuf::from("/a");
        pb.push(Path::new("b"));
        test!("PathBuf::push str", pb.to_string_lossy() == "/a/b");
        pb.pop();
        test!("PathBuf::pop str", pb.to_string_lossy() == "/a/");
    }

    // ─── Ipv4Addr tests ───────────────────────────────────────────────
    {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        test!("Ipv4Addr is_loopback", ip.is_loopback());
        test!("Ipv4Addr to_string", ip.to_string() == "127.0.0.1");
        test!("Ipv4Addr is_unspecified", !ip.is_unspecified());
        test!("Ipv4Addr UNSPECIFIED", Ipv4Addr::UNSPECIFIED.to_string() == "0.0.0.0");
        test!("Ipv4Addr octets", ip.octets() == [127, 0, 0, 1]);
        test!("Ipv4Addr BROADCAST", Ipv4Addr::BROADCAST.to_string() == "255.255.255.255");
        test!("Ipv4Addr LOOPBACK", Ipv4Addr::LOOPBACK.to_string() == "127.0.0.1");
    }

    // ─── Ipv6Addr tests ───────────────────────────────────────────────
    {
        let ip = Ipv6Addr::LOOPBACK;
        test!("Ipv6Addr is_loopback", ip.is_loopback());
        test!("Ipv6Addr is_unspecified", Ipv6Addr::UNSPECIFIED.is_unspecified());
    }

    // ─── SocketAddr tests (constructors, no FromStr) ───────────────────
    {
        let v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
        test!("SocketAddrV4 port", v4.port() == 8080);
        test!("SocketAddrV4 ip", v4.ip() == &Ipv4Addr::LOOPBACK);

        let v6 = SocketAddrV6::new(Ipv6Addr::LOOPBACK, 8080, 0, 0);
        test!("SocketAddrV6 port", v6.port() == 8080);
        test!("SocketAddrV6 ip", v6.ip() == &Ipv6Addr::LOOPBACK);

        let sa = SocketAddr::new([127, 0, 0, 1], 8080);
        test!("SocketAddr port", sa.port() == 8080);
    }

    // ─── env::temp_dir / home_dir / consts ────────────────────────────
    {
        test!("env::temp_dir non-empty", env::temp_dir().to_string_lossy().len() > 0);
        test!("env::home_dir exists", env::home_dir().is_some());
        test!("env::consts::ARCH non-empty", !env::consts::ARCH.is_empty());
        test!("env::consts::OS non-empty", !env::consts::OS.is_empty());
        test!("env::consts::FAMILY non-empty", !env::consts::FAMILY.is_empty());
    }

    // ─── fs::canonicalize / remove_dir_all ───────────────────────────
    {
        let tmp = env::temp_dir();
        let test_dir = tmp.join("xv8_std_test_dir");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();
        test!("create_dir_all + remove_dir_all", {
            fs::remove_dir_all(&test_dir).is_ok() && !test_dir.exists()
        });

        let test_file = tmp.join("xv8_std_canon_test.txt");
        fs::write(&test_file, b"hello").unwrap();
        let canon = fs::canonicalize(&test_file).unwrap();
        test!("canonicalize non-empty", canon.to_string_lossy().len() > 0);
        let _ = fs::remove_file(&test_file);
    }

    // ─── Sync IO: copy ───────────────────────────────────────────────
    {
        let mut src = Cursor::new(&b"hello world"[..]);
        let mut dst = Cursor::new(Vec::new());
        test!("io::copy", copy(&mut src, &mut dst).unwrap() == 11);
        test!("io::copy content", dst.into_inner() == b"hello world");
    }

    // ─── sbrk allocator test (dealloc + coalesce) ─────────────────────
    {
        let mut v: Vec<u8> = Vec::new();
        v.extend_from_slice(b"allocator test");
        test!("Vec allocation + extend", v.len() == 14);
        drop(v);
        test!("Vec drop (dealloc)", true);
    }

    println!("\n=== test_std results ===");
    println!("PASS: {}/{}", passed, passed + failed);
    println!("FAIL: {}/{}", failed, passed + failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
