#![no_std]
#![no_main]

use user::*;

const STATE_DIR: &str = "/var/lib/dock8/containers";

fn fatal(msg: &str) -> ! {
    exit_with_msg(msg)
}

fn container_state_dir(name: &str) -> [u8; 256] {
    let mut buf = [0u8; 256];
    let prefix = b"/var/lib/dock8/containers/";
    buf[..prefix.len()].copy_from_slice(prefix);
    let name_bytes = name.as_bytes();
    let start = prefix.len();
    let end = (start + name_bytes.len()).min(255);
    buf[start..end].copy_from_slice(&name_bytes[..end - start]);
    buf
}

fn write_state(pid: u32, name: &str, cmd: &str) {
    let _ = mkdir("/var");
    let _ = mkdir("/var/lib");
    let _ = mkdir("/var/lib/dock8");
    let _ = mkdir("/var/lib/dock8/containers");
    let dir = container_state_dir(name);
    let _ = mkdir(cstr_as_str(&dir));

    let mut path = [0u8; 256];
    path[..dir.len()].copy_from_slice(&dir[..dir.len()]);
    let plen = dir.len();
    path[plen] = b'/';
    path[plen + 1..plen + 5].copy_from_slice(b"pid\0");
    let fd = open(cstr_as_str(&path), OpenFlag::CREATE | OpenFlag::WRITE_ONLY).unwrap_or_else(|_| fatal("create pid file"));
    let pid_str = [pid as u8];
    write(fd, &pid_str).unwrap();
    close(fd).unwrap();

    path[plen + 1..plen + 9].copy_from_slice(b"command\0");
    let fd = open(cstr_as_str(&path), OpenFlag::CREATE | OpenFlag::WRITE_ONLY).unwrap_or_else(|_| fatal("create cmd file"));
    write(fd, cmd.as_bytes()).unwrap();
    close(fd).unwrap();
}

fn cstr_as_str(buf: &[u8]) -> &str {
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    unsafe { core::str::from_utf8_unchecked(&buf[..len]) }
}

fn cmd_run(args: &[&str]) {
    if args.len() < 2 {
        println!("usage: dock8 run <name> <command> [args...]");
        exit(1);
    }
    let name = args[0];
    let cmd = args[1];
    let cmd_args = &args[1..];

    match fork() {
        Ok(0) => {
            let _ = unshare(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS);

            let _ = sethostname(name.as_bytes());

            let _ = mkdir("/tmp/rootfs");
            let _ = mkdir("/tmp/rootfs/oldroot");
            let _ = mkdir("/tmp/rootfs/dev");
            let _ = mknod("/tmp/rootfs/dev/console", 1, 3);

            if pivot_root("/tmp/rootfs", "/tmp/rootfs/oldroot").is_ok() {
                let _ = chdir("/");
            } else {
                println!("pivot_root failed");
                exit(1);
            }

            let _err = exec(cmd, cmd_args);
            println!("exec failed");
            exit(1);
        }
        Ok(pid) => {
            let name_copy = match args.get(0) {
                Some(n) => n,
                None => "",
            };
            write_state(pid as u32, name_copy, cmd);
            println!("container {} started with pid {}", name_copy, pid);
            let mut code = 0;
            match wait(&mut code) {
                Ok(wpid) if wpid == pid => {
                    println!("container {} exited with code {}", name_copy, code);
                }
                _ => {
                    println!("wait failed for container {}", name_copy);
                }
            }
        }
        Err(_e) => {
            println!("fork failed");
            exit(1);
        }
    }
}

fn cmd_ps() {
    let dir = open(STATE_DIR, OpenFlag::READ_ONLY);
    match dir {
        Ok(fd) => {
            close(fd).unwrap();
        }
        Err(_) => {
            println!("no containers");
            return;
        }
    }
}

fn cmd_exec(args: &[&str]) {
    if args.len() < 2 {
        println!("usage: dock8 exec <name> <command> [args...]");
        exit(1);
    }
    let name = args[0];
    let cmd = args[1];
    let cmd_args = &args[1..];

    let pid_path = {
        let dir = container_state_dir(name);
        let mut path = [0u8; 256];
        let dlen = dir.iter().position(|&b| b == 0).unwrap_or(dir.len());
        path[..dlen].copy_from_slice(&dir[..dlen]);
        path[dlen] = b'/';
        path[dlen + 1..dlen + 5].copy_from_slice(b"pid\0");
        let _ = path;
        dir
    };
    let _ = pid_path;

    let mut pid_buf = [0u8; 1];
    let mut pid_path_buf = [0u8; 256];
    let dir = container_state_dir(name);
    let dlen = dir.iter().position(|&b| b == 0).unwrap_or(dir.len());
    pid_path_buf[..dlen].copy_from_slice(&dir[..dlen]);
    pid_path_buf[dlen] = b'/';
    pid_path_buf[dlen + 1..dlen + 5].copy_from_slice(b"pid\0");
    let pid_str = cstr_as_str(&pid_path_buf);
    let f = match open(pid_str, OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => {
            println!("container {} not found", name);
            exit(1);
        }
    };
    let n = read(f, &mut pid_buf).unwrap_or(0);
    close(f).unwrap_or(());
    if n < 1 {
        println!("container {} has no pid", name);
        exit(1);
    }
    let target_pid = pid_buf[0] as usize;

    match fork() {
        Ok(0) => {
            let ns_fd = match nsopen(target_pid, 5) {
                Ok(fd) => fd,
                Err(_) => {
                    println!("nsopen failed");
                    exit(1);
                }
            };
            let _ = setns(ns_fd, 0);
            let _ = exec(cmd, cmd_args);
            println!("exec failed");
            exit(1);
        }
        Ok(pid) => {
            let mut code = 0;
            let _ = wait(&mut code);
            println!("exec in {} exited with code {}", name, code);
        }
        Err(_) => {
            println!("fork failed");
            exit(1);
        }
    }
}

fn cmd_rm(args: &[&str]) {
    if args.is_empty() {
        println!("usage: dock8 rm <name>");
        exit(1);
    }
    let _name = args[0];
    println!("rm not yet supported");
}

fn cmd_pull(_args: &[&str]) {
    println!("pull not yet supported");
}

#[unsafe(no_mangle)]
fn main(args: Args) {
    let cmd = match args.get_str(1) {
        Some(c) => c,
        None => {
            println!("usage: dock8 <command> [args...]");
            println!("commands: run, exec, ps, rm, pull");
            exit(1);
        }
    };

    let mut cmd_args_vec: [&str; 16] = [""; 16];
    let mut cmd_count = 0;
    for i in 2..args.len() {
        if let Some(arg) = args.get_str(i) {
            if cmd_count < 16 {
                cmd_args_vec[cmd_count] = arg;
                cmd_count += 1;
            }
        }
    }
    let cmd_args = &cmd_args_vec[..cmd_count];

    match cmd {
        "run" => cmd_run(cmd_args),
        "ps" => cmd_ps(),
        "exec" => cmd_exec(cmd_args),
        "rm" => cmd_rm(cmd_args),
        "pull" => cmd_pull(cmd_args),
        _ => {
            println!("unknown command: {}", cmd);
            exit(1);
        }
    }
}
