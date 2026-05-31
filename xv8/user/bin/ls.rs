#![no_std]
#![no_main]

use user::*;

fn type_char(t: InodeType) -> char {
    match t {
        InodeType::File => 'f',
        InodeType::Directory => 'd',
        InodeType::Device => 'D',
        InodeType::SymLink => 'l',
        InodeType::Fifo => 'p',
        InodeType::Free => '?',
    }
}

fn ls(path: &str) {
    let Ok(mut fd) = open(path, OpenFlag::READ_ONLY) else {
        eprintln!("ls: cannot open {}", path);
        return;
    };

    let mut stat = Stat::default();
    if fstat(fd, &mut stat).is_err() {
        eprintln!("ls: cannot stat {}", path);
        let _ = close(fd);
        return;
    }

    match stat.r#type {
        InodeType::Free => {}
        InodeType::Directory => {
            let mut buf = [0u8; size_of::<Directory>()];
            while fd.read(&mut buf) == Ok(buf.len()) {
                let dir: &Directory = unsafe { &*(buf.as_ptr() as *const Directory) };

                if dir.inum == 0 {
                    continue;
                }

                let mut full_path = [0u8; MAXPATH];

                let mut path_len = path.len();
                let name_len = dir
                    .name
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(dir.name.len());

                full_path[..path_len].copy_from_slice(path.as_bytes());
                if !path.ends_with('/') && path_len < MAXPATH - 1 {
                    full_path[path_len] = b'/';
                    path_len += 1;
                }
                if name_len > 0 && path_len + name_len <= MAXPATH {
                    full_path[path_len..path_len + name_len].copy_from_slice(&dir.name[..name_len]);
                }

                let file_path = match core::str::from_utf8(&full_path[..path_len + name_len]) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                ls(file_path);
            }
        }
        InodeType::File | InodeType::Device | InodeType::SymLink | InodeType::Fifo => {
            println!(
                "{} {:>4} {:>8} {}",
                type_char(stat.r#type),
                stat.ino,
                stat.size,
                path
            );
        }
    }

    let _ = close(fd);
}

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.len() < 2 {
        ls(".");
    } else {
        args.args_as_str().for_each(ls);
    }
}