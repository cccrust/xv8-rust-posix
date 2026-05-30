fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut show_all = true;
    let mut show_shm = false;
    let mut show_sem = false;
    let mut show_msg = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => { show_all = true; show_shm = false; show_sem = false; show_msg = false; }
            "-m" => { show_all = false; show_shm = true; }
            "-s" => { show_all = false; show_sem = true; }
            "-q" => { show_all = false; show_msg = true; }
            _ => { show_all = true; }
        }
        i += 1;
    }
    if show_all {
        show_shm = true;
        show_sem = true;
        show_msg = true;
    }
    if show_shm {
        let now = std::process::Command::new("date")
            .arg("+%c").output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        println!("IPC status from <running system> as of {}", now.trim());
        println!("T     ID     KEY        MODE       OWNER    GROUP");
        list_shm();
    }
    if show_sem {
        println!();
        println!("Semaphores:");
        println!("T     ID     KEY        MODE       OWNER    GROUP");
        list_sem();
    }
    if show_msg {
        println!();
        println!("Message Queues: (not supported on this platform)");
    }
}

fn list_shm() {
    for id in 0..256 {
        let mut buf: libc::shmid_ds = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::shmctl(id, libc::IPC_STAT, &mut buf) };
        if ret == 0 {
            let perms = buf.shm_perm;
            let mode_str = format_mode(perms.mode as u16);
            println!("m   {}  {:08x}  {}  {}  {}",
                id, perms._key as u32, mode_str,
                perms.uid, perms.gid);
        }
    }
}

fn list_sem() {
    for id in 0..256 {
        let _buf: libc::semid_ds = unsafe { std::mem::zeroed() };
        unsafe {
            let ret = libc::semctl(id, 0, libc::IPC_STAT, std::ptr::null_mut::<()>());
            if ret >= 0 {
                let mode_str = format_mode(0o644u16);
                println!("s   {}  {:08x}  {}  0  0", id, 0, mode_str);
            }
        }
    }
}

fn format_mode(mode: u16) -> String {
    let m = mode as u32;
    let chars = [
        if m & 0x100 != 0 { 'r' } else { '-' },
        if m & 0x80 != 0 { 'w' } else { '-' },
        if m & 0x40 != 0 { 'x' } else { '-' },
        if m & 0x20 != 0 { 'r' } else { '-' },
        if m & 0x10 != 0 { 'w' } else { '-' },
        if m & 0x8 != 0 { 'x' } else { '-' },
        if m & 0x4 != 0 { 'r' } else { '-' },
        if m & 0x2 != 0 { 'w' } else { '-' },
        if m & 0x1 != 0 { 'x' } else { '-' },
    ];
    chars.iter().collect()
}
