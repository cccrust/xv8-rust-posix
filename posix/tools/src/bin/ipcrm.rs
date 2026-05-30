fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: ipcrm [-m shmid|-M shmkey|-s semid|-S semkey]");
        std::process::exit(1);
    }
    let mut i = 1;
    while i < args.len() {
        if args[i].starts_with('-') && i + 1 < args.len() {
            let flag = &args[i];
            let val = &args[i+1];
            i += 2;
            match flag.as_str() {
                "-m" | "-M" => {
                    let id: i32 = val.parse().unwrap_or(-1);
                    if id >= 0 {
                        unsafe { libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut()); }
                    }
                }
                "-s" | "-S" => {
                    let id: i32 = val.parse().unwrap_or(-1);
                    if id >= 0 {
                        unsafe { libc::semctl(id, 0, libc::IPC_RMID); }
                    }
                }
                _ => {
                    eprintln!("ipcrm: unsupported: {} (only -m/-s on this platform)", flag);
                    std::process::exit(1);
                }
            }
        } else {
            i += 1;
        }
    }
}
