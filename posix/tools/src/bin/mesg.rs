fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        // Report current state
        #[cfg(unix)]
        unsafe {
            let ret = libc::isatty(libc::STDOUT_FILENO);
            if ret != 0 {
                println!("is y");
            } else {
                println!("is n");
            }
        }
        return;
    }
    match args[1].as_str() {
        "y" => {
            #[cfg(unix)]
            unsafe {
                libc::signal(libc::SIGTTOU, libc::SIG_IGN);
            }
        }
        "n" => {
            #[cfg(unix)]
            unsafe {
                libc::signal(libc::SIGTTOU, libc::SIG_DFL);
            }
        }
        _ => {
            eprintln!("mesg: usage: mesg [y|n]");
            std::process::exit(1);
        }
    }
}
