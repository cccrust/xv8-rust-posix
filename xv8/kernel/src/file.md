# 檔案抽象 — file.rs

xv8 的檔案層提供統一的介面來處理不同類型的資源：管道、inode、裝置和網路通訊端。

## 檔案類型

```rust
pub enum FileType {
    None,
    Pipe { pipe: Arc<Pipe> },
    Inode { inode: Inode },
    Device { inode: Inode, major: u16 },
    Socket { socket_id: usize },
    Ping { socket_id: usize },
    TcpSocket { tcp_id: usize },
}
```

## 檔案表

```rust
const NFILE: usize = 64;

pub static FILE_TABLE: FileTable = FileTable::new();

struct FileTable {
    meta: SpinLock<[FileMeta; NFILE]>,      // 參考計數
    inner: [SleepLock<FileInner>; NFILE],  // 檔案狀態
}

struct FileMeta {
    ref_count: usize,
}

struct FileInner {
    readable: bool,
    writeable: bool,
    type: FileType,
    offset: u32,
}

struct File {
    pub id: usize,  // 檔案表索引
}
```

## 檔案配置

```rust
impl File {
    pub fn alloc() -> Result<Self, FsError> {
        let mut meta = FILE_TABLE.meta.lock();

        for (i, meta) in meta.iter_mut().enumerate() {
            if meta.ref_count == 0 {
                meta.ref_count = 1;
                return Ok(Self { id: i });
            }
        }

        err!(FsError::OutOfFile)
    }

    pub fn dup(&mut self) -> Self {
        let meta = &mut FILE_TABLE.meta.lock()[self.id];
        meta.ref_count += 1;
        self.clone()
    }

    pub fn close(&mut self) {
        let mut meta_guard = FILE_TABLE.meta.lock();
        let meta = &mut meta_guard[self.id];

        meta.ref_count -= 1;
        if meta.ref_count > 0 {
            return;
        }

        // 釋放資源
        let inner_copy = {
            let mut inner = FILE_TABLE.inner[self.id].lock();
            let copy = inner.clone();
            meta.ref_count = 0;
            inner.type = FileType::None;
            drop(meta_guard);
            copy
        };

        match inner_copy.type {
            FileType::None => {}
            FileType::Pipe { pipe } => pipe.close(inner_copy.writeable),
            FileType::Inode { inode } | FileType::Device { inode, .. } => inode.put(),
            FileType::Socket { socket_id } => SocketTable::close(socket_id),
            FileType::TcpSocket { tcp_id } => TcpTable::close(tcp_id),
            FileType::Ping { socket_id } => PingTable::close(socket_id),
        }
    }
}
```

## 讀取操作

```rust
impl File {
    pub fn read(&self, addr: VA, n: usize) -> Result<usize, SysError> {
        let mut file_inner = FILE_TABLE.inner[self.id].lock();

        if !file_inner.readable {
            err!(SysError::BadDescriptor);
        }

        match &mut file_inner.type {
            FileType::None => panic!("fileread"),

            FileType::Pipe { pipe } => pipe.read(addr, n),

            FileType::Inode { inode } => {
                let mut inode_inner = inode.lock();

                let dst = unsafe { slice::from_raw_parts_mut(addr.as_mut_ptr(), n) };
                let read = log!(inode.read(&mut inode_inner, file_inner.offset, dst, true));

                if let Ok(read) = read {
                    file_inner.offset += read;
                }

                inode.unlock(inode_inner);
                Ok(read as usize)
            }

            FileType::Device { inode: _, major } => {
                match &DEVICES[*major as usize] {
                    Some(dev) => (dev.read)(addr, n),
                    None => err!(SysError::NoEntry),
                }
            }

            FileType::Socket { .. } | FileType::TcpSocket { .. } | FileType::Ping { .. } => {
                err!(SysError::BadDescriptor)
            }
        }
    }
}
```

## 寫入操作

```rust
impl File {
    pub fn write(&self, addr: VA, n: usize) -> Result<usize, SysError> {
        let mut file_inner = FILE_TABLE.inner[self.id].lock();

        if !file_inner.writeable {
            err!(SysError::BadDescriptor);
        }

        match &mut file_inner.type {
            FileType::None => panic!("filewrite"),

            FileType::Pipe { pipe } => pipe.write(addr, n),

            FileType::Inode { inode } => {
                // 分塊寫入避免超過日誌交易大小
                let max = ((MAXOPBLOCKS - 1 - 1 - 2) / 2) * BSIZE;
                let mut i = 0;

                while i < n {
                    let n1 = (n - i).min(max);

                    let _op = Operation::begin();
                    let mut inode_inner = inode.lock();

                    let src = unsafe {
                        slice::from_raw_parts((addr.as_usize() + i) as *const u8, n1)
                    };
                    let write = log!(inode.write(&mut inode_inner, file_inner.offset, src, true));

                    if let Ok(w) = write {
                        file_inner.offset += w;
                    }

                    inode.unlock(inode_inner);
                    drop(_op);

                    if write.is_err() {
                        break;
                    }

                    i += write.unwrap() as usize;
                }

                if i == n {
                    Ok(n)
                } else {
                    err!(SysError::IoError)
                }
            }

            FileType::Device { inode: _, major } => {
                match &DEVICES[*major as usize] {
                    Some(dev) => (dev.write)(addr, n),
                    None => err!(SysError::NoEntry),
                }
            }

            FileType::Socket { .. } | FileType::TcpSocket { .. } | FileType::Ping { .. } => {
                err!(SysError::InvalidArgument)
            }
        }
    }
}
```

## 檔案偏移 (lseek)

```rust
impl File {
    pub fn lseek(&self, offset: isize, whence: usize) -> Result<isize, SysError> {
        let mut file_inner = FILE_TABLE.inner[self.id].lock();

        match &file_inner.type {
            FileType::None => err!(SysError::BadDescriptor),
            FileType::Pipe { .. } | FileType::Socket { .. } | FileType::Ping { .. }
                | FileType::TcpSocket { .. } => err!(SysError::IsDirectory),

            FileType::Inode { .. } | FileType::Device { .. } => {
                let new_offset = match whence {
                    0 => {  // SEEK_SET
                        if offset < 0 {
                            err!(SysError::InvalidArgument);
                        }
                        offset as u32
                    }
                    1 => {  // SEEK_CUR
                        let base = file_inner.offset as isize;
                        let new = base + offset;
                        if new < 0 {
                            err!(SysError::InvalidArgument);
                        }
                        new as u32
                    }
                    2 => {  // SEEK_END
                        match &file_inner.type {
                            FileType::Inode { inode } => {
                                let inode_inner = inode.lock();
                                let size = inode_inner.size as isize;
                                let new = size + offset;
                                if new < 0 {
                                    err!(SysError::InvalidArgument);
                                }
                                drop(inode_inner);
                                new as u32
                            }
                            _ => err!(SysError::InvalidArgument),
                        }
                    }
                    _ => err!(SysError::InvalidArgument),
                };

                file_inner.offset = new_offset;
                Ok(new_offset as isize)
            }
        }
    }
}
```

## Ioctl 操作

```rust
impl File {
    pub fn ioctl(&self, cmd: usize, arg: usize) -> Result<usize, SysError> {
        let file_inner = FILE_TABLE.inner[self.id].lock();

        match &file_inner.type {
            FileType::Device { major, .. } if *major as usize == CONSOLE => {
                Console::ioctl(cmd, arg)
            }
            FileType::Device { .. } => err!(SysError::NotImplemented),

            FileType::Socket { socket_id } => {
                if cmd == Ioctl::SOCKET_GET_PORT {
                    Ok(SocketTable::get_port_number(*socket_id) as usize)
                } else {
                    err!(SysError::NotImplemented)
                }
            }

            _ => err!(SysError::BadDescriptor),
        }
    }
}
```

## 裝置註冊

```rust
pub struct Device {
    pub read: fn(addr: VA, n: usize) -> Result<usize, SysError>,
    pub write: fn(addr: VA, n: usize) -> Result<usize, SysError>,
}

pub const CONSOLE: usize = 1;

pub static DEVICES: [Option<Device>; NDEV] = {
    let mut devices = [None; NDEV];
    devices[CONSOLE] = Some(Device {
        read: Console::read,
        write: Console::write,
    });
    devices
};
```

## Open 標誌

```rust
pub struct OpenFlag;

impl OpenFlag {
    pub const READ_ONLY: usize = 0x000;
    pub const WRITE_ONLY: usize = 0x001;
    pub const READ_WRITE: usize = 0x002;
    pub const CREATE: usize = 0x040;
    pub const EXCLUSIVE: usize = 0x080;
    pub const TRUNCATE: usize = 0x200;
    pub const APPEND: usize = 0x400;
    pub const NON_BLOCK: usize = 0x800;
}
```

## 相關主題

- [[Pipe]]：管道實作
- [[fs]]：inode 和檔案系統
- [[console]]：主控台裝置