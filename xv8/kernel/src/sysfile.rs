use core::mem;
use core::slice;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use crate::abi::OpenFlag;
use crate::exec::exec;
use crate::file::{FILE_TABLE, File, FileType};
use crate::fs::{BSIZE, Directory, Inode, InodeType, MAXFILE, Path};
use crate::log::Operation;
use crate::param::{MAXARG, MAXPATH, NDEV, NOFILE};
use crate::pipe::Pipe;
use crate::proc;
use crate::proc::{current_proc, current_proc_and_data_mut};
use crate::riscv::PGSIZE;
use crate::spinlock::SpinLock;
use crate::syscall::{SysError, SyscallArgs};
use crate::vm::VA;

/// Allocates a file descriptor for the give file.
/// Takes over file reference from caller on success.
pub fn fd_alloc(file: File) -> Result<usize, SysError> {
    let data = proc::current_proc().data();
    let mut files = data.open_files.as_ref().unwrap().files.lock();

    for (fd, open_file) in files.iter_mut().enumerate() {
        if open_file.is_none() {
            *open_file = Some(file);
            return Ok(fd);
        }
    }

    err!(SysError::TooManyFiles)
}

pub fn sys_dup(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, mut file) = try_log!(args.get_file(0));
    let fd = try_log!(fd_alloc(file.clone()));
    file.dup();
    Ok(fd)
}

pub fn sys_dup2(args: &SyscallArgs) -> Result<usize, SysError> {
    let oldfd = args.get_int(0) as usize;
    let newfd = args.get_int(1) as usize;

    if oldfd >= NOFILE || newfd >= NOFILE {
        err!(SysError::BadDescriptor);
    }

    let data = proc::current_proc().data();
    let mut files = data.open_files.as_ref().unwrap().files.lock();

    let old_file = match files[oldfd].take() {
        Some(f) => f,
        None => return Err(SysError::BadDescriptor),
    };

    if oldfd == newfd {
        files[oldfd] = Some(old_file);
        return Ok(newfd);
    }

    let existing = files[newfd].take();
    let new_file = old_file.dup();
    files[newfd] = Some(new_file);
    files[oldfd] = Some(old_file);
    drop(files);

    if let Some(mut f) = existing {
        f.close();
    }

    Ok(newfd)
}

pub fn sys_read(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(1);
    let n = args.get_int(2);
    let (_, file) = try_log!(args.get_file(0));
    log!(file.read(addr, n as usize))
}

pub fn sys_write(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(1);
    let n = args.get_int(2);
    let (_, mut file) = try_log!(args.get_file(0));
    log!(file.write(addr, n as usize))
}

pub fn sys_close(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;

    if fd >= NOFILE {
        err!(SysError::BadDescriptor);
    }

    let data = proc::current_proc().data();
    let mut files = data.open_files.as_ref().unwrap().files.lock();

    let file = files[fd].take();
    drop(files);

    if let Some(mut f) = file {
        f.close();
    }

    Ok(0)
}

pub fn sys_fstat(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(1);
    let (_, file) = try_log!(args.get_file(0));
    try_log!(file.stat(addr));
    Ok(0)
}

pub fn sys_link(args: &SyscallArgs) -> Result<usize, SysError> {
    let old = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let new = try_log!(args.fetch_string(args.get_addr(1), MAXPATH));

    let _op = Operation::begin();

    // get the inode of the old
    let Ok(old_inode) = log!(Path::new(&old).resolve()) else {
        err!(SysError::NoEntry)
    };

    let mut old_inner = old_inode.lock();

    // make sure it is not a directory
    if old_inner.r#type == InodeType::Directory {
        old_inode.unlock_put(old_inner);
        err!(SysError::NotPermitted);
    }

    // increment number of links pointing to the inode
    old_inner.nlink += 1;
    old_inode.update(&old_inner);
    old_inode.unlock(old_inner);

    // after incrementing nlink, failures must goto `bad`
    let result = (|| {
        // get the inode of the new's parent
        let (parent, name) = match log!(Path::new(&new).resolve_parent()) {
            Ok(v) => v,
            Err(_) => err!(SysError::NoEntry),
        };

        // make sure they are in the same device
        if parent.dev != old_inode.dev {
            err!(SysError::CrossDeviceLink);
        }

        let mut parent_inner = parent.lock();

        // add the inode to the new's parent
        if let Err(e) = log!(Directory::link(
            &parent,
            &mut parent_inner,
            name,
            old_inode.inum as u16
        )) {
            parent.unlock_put(parent_inner);
            err!(SysError::from(e));
        }

        let parent_dev = parent.dev;
        let parent_inum = parent.inum;
        parent.unlock_put(parent_inner);
        crate::inotify::notify(parent_dev, parent_inum, crate::inotify::IN_CREATE, 0, &name);
        Ok(0)
    })();

    // bad
    if result.is_err() {
        let mut old_inner = old_inode.lock();
        old_inner.nlink -= 1;
        old_inode.update(&old_inner);
        old_inode.unlock(old_inner);
    }

    old_inode.put();

    result
}

pub fn sys_unlink(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let _op = Operation::begin();

    // get the parent inode and name
    let Ok((parent, name)) = log!(Path::new(&path).resolve_parent()) else {
        err!(SysError::NoEntry);
    };

    let mut parent_inner = parent.lock();

    // cannot unlink `.` or `..`
    if name == "." || name == ".." {
        parent.unlock_put(parent_inner);
        err!(SysError::InvalidArgument);
    }

    // find the inode in the parent's directory entry
    let Ok(Some((offset, inode))) = log!(Directory::lookup(&parent, &mut parent_inner, name))
    else {
        parent.unlock_put(parent_inner);
        err!(SysError::NoEntry);
    };

    let mut inode_inner = inode.lock();

    assert!(inode_inner.nlink >= 1, "unlink nlink < 1");

    // if the inode is a directory and it is not empty, cannot unlink
    if inode_inner.r#type == InodeType::Directory && !Directory::is_empty(&inode, &mut inode_inner)
    {
        inode.unlock_put(inode_inner);
        parent.unlock_put(parent_inner);
        err!(SysError::NotEmpty);
    }

    // replace the directory entry with an empty one
    let dir = Directory::new_empty();
    match log!(parent.write(&mut parent_inner, offset, dir.as_bytes(), false)) {
        Ok(write) => {
            assert_eq!(write, Directory::SIZE as u32, "unlink write");
        }
        Err(_) => {
            parent.unlock_put(parent_inner);
            err!(SysError::IoError)
        }
    }

    // if it is a directory, decrement parent's link count
    if inode_inner.r#type == InodeType::Directory {
        parent_inner.nlink -= 1;
        parent.update(&parent_inner);
    }
    let parent_dev = parent.dev;
    let parent_inum = parent.inum;
    parent.unlock_put(parent_inner);

    // decrement the inode's link count
    inode_inner.nlink -= 1;
    inode.update(&inode_inner);
    inode.unlock_put(inode_inner);

    crate::inotify::notify(parent_dev, parent_inum, crate::inotify::IN_DELETE, 0, &name);

    Ok(0)
}

pub fn sys_open(args: &SyscallArgs) -> Result<usize, SysError> {
    let o_mode = args.get_int(1) as usize;
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let path = Path::new(&path);

    let _op = Operation::begin();

    let (mut inode, mut inode_inner);

    // either create a new file or find the file from the path
    if (o_mode & OpenFlag::CREATE) != 0 {
        (inode, inode_inner) = match log!(Inode::create(&path, InodeType::File, 0, 0)) {
            Ok(i) => i,
            Err(e) => {
                err!(SysError::from(e))
            }
        };
        if let Ok((parent, child_name)) = path.resolve_parent() {
            crate::inotify::notify(parent.dev, parent.inum, crate::inotify::IN_CREATE, 0, &child_name);
            parent.put();
        }
    } else {
        inode = match log!(path.resolve()) {
            Ok(i) => i,
            Err(_) => {
                err!(SysError::NoEntry);
            }
        };

        inode_inner = inode.lock();

        // if it is a directory, cannot open with write mode
        if inode_inner.r#type == InodeType::Directory && o_mode != OpenFlag::READ_ONLY {
            inode.unlock_put(inode_inner);
            err!(SysError::IsDirectory);
        }
    }

    // cannot open device out of range
    if inode_inner.r#type == InodeType::Device && inode_inner.major >= NDEV as u16 {
        inode.unlock_put(inode_inner);
        err!(SysError::NoEntry);
    }

    // allocate a file structure and a file descriptor
    let (fd, file) = match log!(File::alloc()) {
        Ok(mut file) => match log!(fd_alloc(file.clone())) {
            Ok(fd) => (fd, file),
            Err(e) => {
                // if err here, we must also close the file
                file.close();
                inode.unlock_put(inode_inner);
                return Err(e);
            }
        },
        Err(e) => {
            inode.unlock_put(inode_inner);
            err!(SysError::from(e));
        }
    };

    let mut file_inner = FILE_TABLE.inner[file.id].lock();
    if inode_inner.r#type == InodeType::Device {
        file_inner.r#type = FileType::Device {
            inode: inode.clone(),
            major: inode_inner.major,
        };
    } else if inode_inner.r#type == InodeType::Fifo {
        let inum = inode.inum;
        let pipe = {
            let mut fifo = FIFO_TABLE.lock();
            if let Some((_, p)) = fifo.iter().find(|(i, _)| *i == inum) {
                p.clone()
            } else {
                let pipe = match log!(Pipe::alloc_arc()) {
                    Ok(p) => p,
                    Err(e) => {
                        inode.unlock_put(inode_inner);
                        return Err(SysError::from(e));
                    }
                };
                fifo.push((inum, pipe.clone()));
                pipe
            }
        };
        file_inner.r#type = FileType::Pipe { pipe };
        file_inner.readable = (o_mode & OpenFlag::WRITE_ONLY) == 0;
        file_inner.writeable =
            (o_mode & OpenFlag::WRITE_ONLY) != 0 || (o_mode & OpenFlag::READ_WRITE != 0);
    } else {
        file_inner.r#type = FileType::Inode {
            inode: inode.clone(),
        };
        file_inner.offset = 0;
    }
    file_inner.readable = (o_mode & OpenFlag::WRITE_ONLY) == 0;
    file_inner.writeable =
        (o_mode & OpenFlag::WRITE_ONLY) != 0 || (o_mode & OpenFlag::READ_WRITE != 0);
    file_inner.nonblocking = (o_mode & OpenFlag::NON_BLOCK) != 0;

    if (o_mode & OpenFlag::TRUNCATE) != 0 && inode_inner.r#type == InodeType::File {
        inode.trunc(&mut inode_inner);
        crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_MODIFY, 0, "");
    }

    inode.unlock(inode_inner);

    crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_OPEN, 0, "");

    Ok(fd)
}

pub fn sys_mkdir(args: &SyscallArgs) -> Result<usize, SysError> {
    let _op = Operation::begin();

    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let (inode, inode_inner) =
        match log!(Inode::create(&Path::new(&path), InodeType::Directory, 0, 0)) {
            Ok(i) => i,
            Err(e) => err!(SysError::from(e)),
        };

    if let Ok((parent, child_name)) = Path::new(&path).resolve_parent() {
        crate::inotify::notify(parent.dev, parent.inum, crate::inotify::IN_CREATE | crate::inotify::IN_ISDIR, 0, &child_name);
        parent.put();
    }

    inode.unlock_put(inode_inner);

    Ok(0)
}

pub fn sys_mknod(args: &SyscallArgs) -> Result<usize, SysError> {
    let _op = Operation::begin();

    let major = args.get_int(1) as u16;
    let minor = args.get_int(2) as u16;
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let (inode, inner) = match log!(Inode::create(
        &Path::new(&path),
        InodeType::Device,
        major,
        minor,
    )) {
        Ok(i) => i,
        Err(e) => err!(SysError::from(e)),
    };

    inode.unlock_put(inner);

    Ok(0)
}

pub fn sys_chdir(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_proc, data) = current_proc_and_data_mut();

    let _op = Operation::begin();

    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let Ok(inode) = log!(Path::new(&path).resolve()) else {
        err!(SysError::NoEntry);
    };

    let inner = inode.lock();

    if inner.r#type != InodeType::Directory {
        inode.unlock_put(inner);
        err!(SysError::NotDirectory);
    }

    inode.unlock(inner);

    let old_cwd = mem::replace(&mut data.cwd, inode);
    old_cwd.put();

    Ok(0)
}

pub fn sys_exec(args: &SyscallArgs) -> Result<usize, SysError> {
    let uargv = args.get_addr(1);

    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let path = Path::new(&path);

    let (_proc, data) = current_proc_and_data_mut();

    let mut argv_bufs: Vec<String> = Vec::with_capacity(MAXARG);

    for i in 0..MAXARG {
        // fetch pointer argv[i] from user space
        let mut uarg: usize = 0;
        let dst = unsafe {
            slice::from_raw_parts_mut(&mut uarg as *mut usize as *mut u8, size_of::<usize>())
        };
        if log!(
            data.pagetable_mut()
                .copy_from(uargv + i * size_of::<usize>(), dst)
        )
        .is_err()
        {
            err!(SysError::BadAddress);
        }

        if uarg == 0 {
            break; // NULL terminator
        }

        // fetch string from user space
        let s = try_log!(args.fetch_string(VA::from(uarg), PGSIZE));
        argv_bufs.push(s);
    }

    let argv: Vec<&str> = argv_bufs.iter().map(|s| s.as_str()).collect::<Vec<_>>();

    log!(exec(&path, &argv)).map_err(|_| SysError::InvalidExecutable)
}

pub fn sys_pipe(args: &SyscallArgs) -> Result<usize, SysError> {
    // user pointer to array of two integers
    let fd_array = args.get_addr(0);

    let (_proc, data) = current_proc_and_data_mut();

    let (mut read, mut write) = match log!(Pipe::alloc()) {
        Ok(pair) => pair,
        Err(e) => err!(SysError::from(e)),
    };

    let Ok(fd0) = log!(fd_alloc(read.clone())) else {
        read.close();
        write.close();
        err!(SysError::TooManyFiles);
    };

    let Ok(fd1) = log!(fd_alloc(write.clone())) else {
        let mut files = data.open_files.as_ref().unwrap().files.lock();
        files[fd0] = None;
        drop(files);
        read.close();
        write.close();
        err!(SysError::TooManyFiles);
    };

    let pagetable = data.pagetable_mut();

    if log!(pagetable.copy_to(&fd0.to_le_bytes(), fd_array)).is_err()
        || log!(pagetable.copy_to(&fd1.to_le_bytes(), fd_array + size_of_val(&fd1))).is_err()
    {
        let mut files = data.open_files.as_ref().unwrap().files.lock();
        files[fd0] = None;
        files[fd1] = None;
        drop(files);
        read.close();
        write.close();
        err!(SysError::BadAddress);
    }

    Ok(0)
}

pub fn sys_ioctl(args: &SyscallArgs) -> Result<usize, SysError> {
    let ioctl_cmd = args.get_int(1) as usize;
    let ioctl_arg = args.get_int(2) as usize;
    let (_, file) = try_log!(args.get_file(0));
    log!(file.ioctl(ioctl_cmd, ioctl_arg))
}

pub fn sys_lseek(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;
    let offset = args.get_int(1) as isize;
    let whence = args.get_int(2) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_, file) = try_log!(args.get_file(0));
    file.lseek(offset, whence).map(|v| v as usize)
}

pub fn sys_truncate(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let _op = Operation::begin();
    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(SysError::NoEntry),
    };

    let mut file = {
        let f = match log!(File::alloc()) {
            Ok(f) => f,
            Err(e) => {
                inode.put();
                err!(SysError::from(e));
            }
        };
        let mut file_inner = FILE_TABLE.inner[f.id].lock();
        file_inner.r#type = FileType::Inode { inode: inode.clone() };
        file_inner.readable = false;
        file_inner.writeable = true;
        f
    };

    let inode_dev = inode.dev;
    let inode_inum = inode.inum;
    drop(inode);
    log!(file.truncate(0))?;
    crate::inotify::notify(inode_dev, inode_inum, crate::inotify::IN_MODIFY, 0, "");
    file.close();
    Ok(0)
}

pub fn sys_ftruncate(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, file) = try_log!(args.get_file(0));
    log!(file.truncate(0))?;
    let inner = crate::file::FILE_TABLE.inner[file.id].lock();
    if let crate::file::FileType::Inode { inode } = &inner.r#type {
        crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_MODIFY, 0, "");
    }
    drop(inner);
    Ok(0)
}

pub fn sys_chmod(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let mode = args.get_int(1) as u16;

    let _op = Operation::begin();
    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(SysError::NoEntry),
    };

    let mut file = {
        let f = match log!(File::alloc()) {
            Ok(f) => f,
            Err(e) => {
                inode.put();
                err!(SysError::from(e));
            }
        };
        let mut file_inner = FILE_TABLE.inner[f.id].lock();
        file_inner.r#type = FileType::Inode { inode: inode.clone() };
        file_inner.readable = false;
        file_inner.writeable = false;
        f
    };

    let inode_dev = inode.dev;
    let inode_inum = inode.inum;
    drop(inode);
    log!(file.chmod(mode))?;
    crate::inotify::notify(inode_dev, inode_inum, crate::inotify::IN_ATTRIB, 0, "");
    file.close();
    Ok(0)
}

pub fn sys_fchmod(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, file) = try_log!(args.get_file(0));
    let mode = args.get_int(1) as u16;
    log!(file.chmod(mode))?;
    let inner = crate::file::FILE_TABLE.inner[file.id].lock();
    if let crate::file::FileType::Inode { inode } = &inner.r#type {
        crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_ATTRIB, 0, "");
    }
    drop(inner);
    Ok(0)
}

pub fn sys_chown(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let uid = args.get_int(1) as u16;
    let gid = args.get_int(2) as u16;

    let _op = Operation::begin();
    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(SysError::NoEntry),
    };

    let mut file = {
        let f = match log!(File::alloc()) {
            Ok(f) => f,
            Err(e) => {
                inode.put();
                err!(SysError::from(e));
            }
        };
        let mut file_inner = FILE_TABLE.inner[f.id].lock();
        file_inner.r#type = FileType::Inode { inode: inode.clone() };
        file_inner.readable = false;
        file_inner.writeable = false;
        f
    };

    let inode_dev = inode.dev;
    let inode_inum = inode.inum;
    drop(inode);
    log!(file.chown(uid, gid))?;
    crate::inotify::notify(inode_dev, inode_inum, crate::inotify::IN_ATTRIB, 0, "");
    file.close();
    Ok(0)
}

pub fn sys_fchown(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, file) = try_log!(args.get_file(0));
    let uid = args.get_int(1) as u16;
    let gid = args.get_int(2) as u16;
    log!(file.chown(uid, gid))?;
    let inner = crate::file::FILE_TABLE.inner[file.id].lock();
    if let crate::file::FileType::Inode { inode } = &inner.r#type {
        crate::inotify::notify(inode.dev, inode.inum, crate::inotify::IN_ATTRIB, 0, "");
    }
    drop(inner);
    Ok(0)
}

pub fn sys_access(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let _mode = args.get_int(1) as u16;

    let _op = Operation::begin();
    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(SysError::NoEntry),
    };
    inode.put();
    Ok(0)
}

pub fn sys_rename(args: &SyscallArgs) -> Result<usize, SysError> {
    let old = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let new = try_log!(args.fetch_string(args.get_addr(1), MAXPATH));

    let _op = Operation::begin();

    let old_inode = match log!(Path::new(&old).resolve()) {
        Ok(i) => i,
        Err(_) => err!(SysError::NoEntry),
    };

    let (parent_new, name_new) = match log!(Path::new(&new).resolve_parent()) {
        Ok(v) => v,
        Err(_) => {
            old_inode.put();
            err!(SysError::NoEntry);
        }
    };

    if old_inode.dev != parent_new.dev {
        old_inode.put();
        parent_new.put();
        err!(SysError::CrossDeviceLink);
    }

    let mut parent_new_inner = parent_new.lock();

    if let Err(e) = log!(Directory::link(
        &parent_new,
        &mut parent_new_inner,
        name_new,
        old_inode.inum as u16
    )) {
        parent_new.unlock_put(parent_new_inner);
        old_inode.put();
        err!(SysError::from(e));
    }

    let cookie = old_inode.inum;
    let parent_new_dev = parent_new.dev;
    let parent_new_inum = parent_new.inum;
    parent_new.unlock_put(parent_new_inner);

    drop(old_inode);

    let name_old_str = match log!(Path::new(&old).resolve_parent()) {
        Ok((_, n)) => n,
        Err(_) => err!(SysError::NoEntry),
    };

    let parent_old = match log!(Path::new(&old).resolve_parent()) {
        Ok((p, _)) => p,
        Err(_) => err!(SysError::NoEntry),
    };

    let parent_old_dev = parent_old.dev;
    let parent_old_inum = parent_old.inum;
    let mut parent_old_inner = parent_old.lock();

    let dir = Directory::new_empty();
    if let Err(_) = log!(parent_old.write(&mut parent_old_inner, 0, dir.as_bytes(), false)) {
        parent_old.unlock_put(parent_old_inner);
        err!(SysError::IoError);
    }

    parent_old.unlock_put(parent_old_inner);

    crate::inotify::notify(parent_new_dev, parent_new_inum, crate::inotify::IN_MOVED_TO, cookie, &name_new);
    crate::inotify::notify(parent_old_dev, parent_old_inum, crate::inotify::IN_MOVED_FROM, cookie, &name_old_str);

    Ok(0)
}

pub fn sys_symlink(args: &SyscallArgs) -> Result<usize, SysError> {
    let target = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let path = try_log!(args.fetch_string(args.get_addr(1), MAXPATH));

    let _op = Operation::begin();

    let (inode, mut inode_inner) = match log!(Inode::create(
        &Path::new(&path),
        InodeType::SymLink,
        0,
        0
    )) {
        Ok(i) => i,
        Err(e) => err!(SysError::from(e)),
    };

    let max_size = (MAXFILE * BSIZE) as u32;
    if target.len() as u32 > max_size {
        inode.unlock_put(inode_inner);
        err!(SysError::NameTooLong);
    }

    let written = log!(inode.write(&mut inode_inner, 0, target.as_bytes(), false));
    if written.is_err() || written.unwrap() != target.len() as u32 {
        inode.unlock_put(inode_inner);
        err!(SysError::IoError);
    }

    inode.unlock_put(inode_inner);
    Ok(0)
}

pub fn sys_readlink(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let buf_addr = args.get_addr(1);
    let bufsiz = args.get_int(2) as usize;

    let _op = Operation::begin();

    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(SysError::NoEntry),
    };

    {
        let inode_inner = inode.lock();
        if inode_inner.r#type != InodeType::SymLink {
            inode.unlock_put(inode_inner);
            err!(SysError::InvalidArgument);
        }
    }

    let mut buf = vec![0u8; bufsiz.min(256)];
    let n = {
        let inode = inode.clone();
        let mut inode_inner = inode.lock();
        inode.read(&mut inode_inner, 0, &mut buf, false)?
    };

    inode.put();

    try_log!(proc::copy_to_user(&buf[..n as usize], buf_addr).map_err(|_| SysError::BadAddress));

    Ok(n as usize)
}

#[allow(dead_code)]
pub struct Timespec {
    pub sec: u32,
    pub nsec: u32,
}

pub fn sys_utimensat(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let times_addr = args.get_addr(1);
    let flags = args.get_int(2) as usize;

    if times_addr == 0 {
        return Err(SysError::BadAddress);
    }

    let now = crate::trap::TICKS.lock();
    let current_time = *now;
    drop(now);

    let _op = Operation::begin();

    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(SysError::NoEntry),
    };

    if flags & 0x10000 != 0 {
        // AT_SYMLINK_NOFOLLOW - don't follow symlinks
    }

    {
        let mut inode_inner = inode.lock();
        inode_inner.atime = current_time as u32;
        inode_inner.mtime = current_time as u32;
        inode.update(&inode_inner);
    }

    let inode_dev = inode.dev;
    let inode_inum = inode.inum;
    inode.put();
    crate::inotify::notify(inode_dev, inode_inum, crate::inotify::IN_ATTRIB, 0, "");
    Ok(0)
}

pub fn sys_readv(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;
    let iov_addr = args.get_addr(1);
    let iovcnt = args.get_int(2) as usize;

    if iovcnt == 0 {
        return Ok(0);
    }

    let (_, file) = try_log!(args.get_file(fd));
    let mut total = 0;

    let (_proc, data) = current_proc_and_data_mut();
    let pt = data.pagetable_mut();

    for i in 0..iovcnt {
        let iovec_addr = iov_addr + i * 16;
        let mut buf_ptr_bytes = [0u8; 8];
        let mut buf_len_bytes = [0u8; 8];
        if pt.copy_from(iovec_addr, &mut buf_ptr_bytes).is_err() {
            break;
        }
        if pt.copy_from(iovec_addr + 8, &mut buf_len_bytes).is_err() {
            break;
        }
        let buf_ptr = usize::from_le_bytes(buf_ptr_bytes);
        let buf_len = usize::from_le_bytes(buf_len_bytes);

        if buf_ptr == 0 || buf_len == 0 {
            continue;
        }

        let addr = VA::from(buf_ptr);
        match file.read(addr, buf_len) {
            Ok(n) => {
                total += n;
                if n < buf_len {
                    break;
                }
            }
            Err(e) => {
                if total > 0 {
                    return Ok(total);
                }
                return Err(e);
            }
        }
    }

    Ok(total)
}

pub fn sys_writev(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;
    let iov_addr = args.get_addr(1);
    let iovcnt = args.get_int(2) as usize;

    if iovcnt == 0 {
        return Ok(0);
    }

    let (_, mut file) = try_log!(args.get_file(fd));
    let mut total = 0;

    let (_proc, data) = current_proc_and_data_mut();
    let pt = data.pagetable_mut();

    for i in 0..iovcnt {
        let iovec_addr = iov_addr + i * 16;
        let mut buf_ptr_bytes = [0u8; 8];
        let mut buf_len_bytes = [0u8; 8];
        if pt.copy_from(iovec_addr, &mut buf_ptr_bytes).is_err() {
            break;
        }
        if pt.copy_from(iovec_addr + 8, &mut buf_len_bytes).is_err() {
            break;
        }
        let buf_ptr = usize::from_le_bytes(buf_ptr_bytes);
        let buf_len = usize::from_le_bytes(buf_len_bytes);

        if buf_ptr == 0 || buf_len == 0 {
            continue;
        }

        let addr = VA::from(buf_ptr);
        match file.write(addr, buf_len) {
            Ok(n) => {
                total += n;
                if n < buf_len {
                    break;
                }
            }
            Err(e) => {
                if total > 0 {
                    return Ok(total);
                }
                return Err(e);
            }
        }
    }

    Ok(total)
}

pub fn sys_pread(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;
    let addr = args.get_addr(1);
    let n = args.get_int(2) as usize;
    let offset = args.get_int(3) as isize;

    let (_, file) = try_log!(args.get_file(fd));

    match &mut FILE_TABLE.inner[file.id].lock().r#type {
        FileType::Inode { inode } => {
            let inode = inode.clone();
            let mut inode_inner = inode.lock();
            let dst = unsafe { slice::from_raw_parts_mut(addr.as_mut_ptr(), n) };
            let read = log!(inode.read(&mut inode_inner, offset as u32, dst, true));
            drop(inode_inner);
            inode.put();
            read.map(|r| r as usize).map_err(|_| SysError::IoError)
        }
        _ => Err(SysError::BadDescriptor),
    }
}

pub fn sys_pwrite(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;
    let addr = args.get_addr(1);
    let n = args.get_int(2) as usize;
    let offset = args.get_int(3) as isize;

    let (_, mut file) = try_log!(args.get_file(fd));

    match &mut FILE_TABLE.inner[file.id].lock().r#type {
        FileType::Inode { inode } => {
            let inode = inode.clone();
            let mut inode_inner = inode.lock();
            let src = unsafe { slice::from_raw_parts(addr.as_mut_ptr() as *const u8, n) };
            let written = log!(inode.write(&mut inode_inner, offset as u32, src, true));
            drop(inode_inner);
            inode.put();
            written.map(|w| w as usize).map_err(|_| SysError::IoError)
        }
        _ => Err(SysError::BadDescriptor),
    }
}

const F_GETFL: isize = 3;
const F_SETFL: isize = 4;

pub fn sys_fcntl(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;
    let cmd = args.get_int(1);
    let arg = args.get_int(2) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_, file) = try_log!(args.get_file(0));
    let mut inner = FILE_TABLE.inner[file.id].lock();

    match cmd {
        F_GETFL => {
            let flags = if inner.nonblocking { OpenFlag::NON_BLOCK } else { 0 };
            Ok(flags)
        }
        F_SETFL => {
            let nb = (arg & OpenFlag::NON_BLOCK) != 0;
            inner.nonblocking = nb;
            if let crate::file::FileType::TcpSocket { tcp_id } = inner.r#type {
                let mut tcp_table = crate::net::tcp::TCP_TABLE.lock();
                if let Some(ref mut conn) = tcp_table.entries[tcp_id] {
                    conn.nonblocking = nb;
                }
            }
            Ok(0)
        }
        _ => err!(SysError::InvalidArgument),
    }
}

static FIFO_TABLE: SpinLock<Vec<(u32, Arc<Pipe>)>> = SpinLock::new(Vec::new(), "fifo");

pub fn sys_mkfifo(args: &SyscallArgs) -> Result<usize, SysError> {
    let _op = Operation::begin();

    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let (inode, inner) = match log!(Inode::create(&Path::new(&path), InodeType::Fifo, 0, 0)) {
        Ok(i) => i,
        Err(e) => err!(SysError::from(e)),
    };

    inode.unlock_put(inner);

    Ok(0)
}

pub fn sys_pipe2(args: &SyscallArgs) -> Result<usize, SysError> {
    let _flags = args.get_int(0) as usize;
    sys_pipe(args)
}

/// splice(fd_in, off_in, fd_out, off_out, len, flags)
/// Move data between two file descriptors without going through user space.
/// At least one FD must be a pipe.
pub fn sys_splice(args: &SyscallArgs) -> Result<usize, SysError> {
    let _fd_in_val = args.get_int(0);
    let _off_in = args.get_addr(1);
    let _fd_out_val = args.get_int(2);
    let _off_out = args.get_addr(3);
    let len = args.get_int(4) as usize;
    let _flags = args.get_int(5) as u32;

    let (_, file_in) = try_log!(args.get_file(0));
    let (_, mut file_out) = try_log!(args.get_file(2));

    let pipe_in = file_in.get_pipe();
    let pipe_out = file_out.get_pipe();

    // If both are pipes: read from src, write to dst via kernel buffer
    if let Ok(in_pipe) = pipe_in {
        if let Ok(out_pipe) = pipe_out {
            let mut buf = vec![0u8; len];
            let n = try_log!(in_pipe.read_kernel(&mut buf));
            if n == 0 {
                return Ok(0);
            }
            let written = try_log!(out_pipe.write_kernel(&buf[..n]));
            return Ok(written);
        }
        // pipe → non-pipe: read from pipe, write to other fd
        let mut buf = vec![0u8; len];
        let n = try_log!(in_pipe.read_kernel(&mut buf));
        if n == 0 {
            return Ok(0);
        }
        // Allocate a page to serve as user-space VA for file_out.write
        let page = try_log!(crate::kalloc::alloc_page().ok_or(SysError::ResourceUnavailable));
        let va = page as usize;
        unsafe {
            core::ptr::copy_nonoverlapping(buf.as_ptr(), va as *mut u8, n);
        }
        let result = file_out.write(VA::from(va), n);
        unsafe { core::ptr::write_volatile(page, 0); }
        crate::kalloc::free_page(page);
        return result;
    }
    if let Ok(out_pipe) = pipe_out {
        // non-pipe → pipe: read from other fd, write to pipe
        let page = try_log!(crate::kalloc::alloc_page().ok_or(SysError::ResourceUnavailable));
        let va = VA::from(page as usize);
        let n = try_log!(file_in.read(va, len));
        if n == 0 {
            unsafe { core::ptr::write_volatile(page, 0); }
            crate::kalloc::free_page(page);
            return Ok(0);
        }
        let mut buf = vec![0u8; n];
        unsafe {
            core::ptr::copy_nonoverlapping(page as *const u8, buf.as_mut_ptr(), n);
        }
        let written = try_log!(out_pipe.write_kernel(&buf));
        unsafe { core::ptr::write_volatile(page, 0); }
        crate::kalloc::free_page(page);
        return Ok(written);
    }
    Err(SysError::BadDescriptor)
}

/// tee(fd_in, fd_out, len, flags)
/// Copy data between two pipes. Data is consumed from fd_in.
pub fn sys_tee(args: &SyscallArgs) -> Result<usize, SysError> {
    let len = args.get_int(2) as usize;
    let _flags = args.get_int(3) as u32;

    let (_, file_in) = try_log!(args.get_file(0));
    let (_, file_out) = try_log!(args.get_file(1));

    let in_pipe = try_log!(file_in.get_pipe());
    let out_pipe = try_log!(file_out.get_pipe());

    let mut buf = vec![0u8; len];
    let n = try_log!(in_pipe.read_kernel(&mut buf));
    if n == 0 {
        return Ok(0);
    }
    let written = try_log!(out_pipe.write_kernel(&buf[..n]));
    Ok(written)
}

/// vmsplice(fd, iov, nr_segs, flags)
/// Copy user pages to a pipe.
pub fn sys_vmsplice(args: &SyscallArgs) -> Result<usize, SysError> {
    let _fd_val = args.get_int(0);
    let iov_addr = args.get_addr(1);
    let nr_segs = args.get_int(2) as usize;
    let _flags = args.get_int(3) as u32;

    let (_, file) = try_log!(args.get_file(0));
    let pipe = try_log!(file.get_pipe());

    let (_proc, data) = current_proc_and_data_mut();
    let pt = data.pagetable_mut();

    let mut total = 0;

    for i in 0..nr_segs {
        if total >= 32768 {
            break;
        }
        let iovec_addr = iov_addr + i * 16;
        let mut buf_ptr_bytes = [0u8; 8];
        let mut buf_len_bytes = [0u8; 8];
        if pt.copy_from(iovec_addr, &mut buf_ptr_bytes).is_err() {
            break;
        }
        if pt.copy_from(iovec_addr + 8, &mut buf_len_bytes).is_err() {
            break;
        }
        let buf_ptr = usize::from_le_bytes(buf_ptr_bytes);
        let buf_len = usize::from_le_bytes(buf_len_bytes);

        if buf_ptr == 0 || buf_len == 0 {
            continue;
        }

        let len = buf_len.min(32768 - total);
        let mut buf = vec![0u8; len];
        if pt.copy_from(VA::from(buf_ptr), &mut buf).is_err() {
            break;
        }
        let n = try_log!(pipe.write_kernel(&buf));
        total += n;
        if n < len {
            break;
        }
    }

    if total == 0 {
        err!(SysError::BadAddress);
    }
    Ok(total)
}
