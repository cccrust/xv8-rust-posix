# 系統呼叫 — syscall.rs

系統呼叫是使用者程式請求核心服務的介面。xv8 實現了約 50 個系統呼叫。

## 系統呼叫觸發

```rust
// 使用者空間
ecall  // 觸發環境呼叫

// 進入核心後
usertrap() → scause::Exception::EnvironmentCall → syscall()
```

## 呼叫編號

```rust
pub enum Syscall {
    Fork = 1,
    Exit = 2,
    Wait = 3,
    Pipe = 4,
    Read = 5,
    Write = 6,
    Kill = 7,
    Exec = 8,
    Open = 9,
    // ...
}
```

## 參數傳遞

RISC-V 上系統呼叫透過暫存器傳遞參數：

```rust
struct SyscallArgs<'a> {
    trapframe: &'a TrapFrame,
    proc: &'static Proc,
}

impl SyscallArgs {
    fn get_raw(&self, index: usize) -> usize {
        match index {
            0 => self.trapframe.a0,
            1 => self.trapframe.a1,
            2 => self.trapframe.a2,
            3 => self.trapframe.a3,
            4 => self.trapframe.a4,
            5 => self.trapframe.a5,
            _ => panic!("invalid syscall argument"),
        }
    }

    fn get_int(&self, index: usize) -> isize {
        self.get_raw(index) as isize
    }

    fn get_addr(&self, index: usize) -> VA {
        VA::from(self.get_raw(index))
    }
}
```

## 錯誤處理

```rust
pub enum SysError {
    NotPermitted = 1,
    NoEntry = 2,
    NoProcess = 3,
    IoError = 5,
    BadDescriptor = 9,
    OutOfMemory = 12,
    // ...
}

impl SysError {
    pub fn as_code(self) -> u16 {
        self as u16
    }

    pub fn from_code(code: u16) -> Self {
        match code {
            1 => Self::NotPermitted,
            2 => Self::NoEntry,
            // ...
            _ => Self::InvalidArgument,
        }
    }
}
```

## 系統呼叫分派

```rust
pub unsafe fn syscall(trapframe: &mut TrapFrame) {
    let proc = current_proc();
    let args = SyscallArgs::new(trapframe, proc);

    let result = match Syscall::try_from(trapframe.a7) {
        Ok(syscall) => match syscall {
            Syscall::Fork => sys_fork(&args),
            Syscall::Exit => sys_exit(&args),
            Syscall::Wait => sys_wait(&args),
            Syscall::Read => sys_read(&args),
            Syscall::Write => sys_write(&args),
            Syscall::Open => sys_open(&args),
            // ... 其餘系統呼叫
        },
        Err(e) => Err(e),
    };

    // 錯誤時返回負值
    trapframe.a0 = match log!(result) {
        Ok(v) => v,
        Err(error) => -(error.as_code() as isize) as usize,
    };
}
```

## 程序相關系統呼叫

### fork

```rust
pub fn sys_fork(args: &SyscallArgs) -> Result<Pid, SysError> {
    proc::fork()
}
```

### exec

```rust
pub fn sys_exec(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = args.fetch_string(args.get_addr(0), 128)?;
    // 解析 argv
    exec::exec(path, argv)
}
```

### wait

```rust
pub fn sys_wait(args: &SyscallArgs) -> Result<Option<Pid>, SysError> {
    let addr = VA::from(args.get_raw(0));
    Ok(proc::wait(addr).map(|pid| *pid))
}
```

### exit

```rust
pub fn sys_exit(args: &SyscallArgs) -> Result<!, SysError> {
    let status = args.get_int(0);
    proc::exit(status);
}
```

## 檔案相關系統呼叫

### open

```rust
pub fn sys_open(args: &SyscallArgs) -> Result<usize, SysError> {
    let path = args.fetch_string(args.get_addr(0), 128)?;
    let mode = args.get_raw(1);

    let (inode, inner) = log!(fs::Path::new(&path).resolve())?;
    let mut inner = inode.lock();

    // 根據 mode 建立或開啟
    if mode & CREATE != 0 {
        drop(inner);
        let (inode, inner) = log!(fs::Inode::create(
            fs::Path::new(&path),
            fs::InodeType::File,
            0,
            0
        ))?;
        let mut inner = inode.lock();
        // ...
    }

    // 分配檔案描述符
    let mut file = log!(File::alloc())?;
    FILE_TABLE.inner[file.id].lock().type = FileType::Inode { inode: inode.dup() };
    Ok(file.id)
}
```

### read/write

```rust
pub fn sys_read(args: &SyscallArgs) -> Result<usize, SysError> {
    let (fd, mut file) = args.get_file(0)?;
    let addr = args.get_addr(1);
    let n = args.get_raw(2);

    file.read(addr, n)
}

pub fn sys_write(args: &SyscallArgs) -> Result<usize, SysError> {
    let (fd, mut file) = args.get_file(0)?;
    let addr = args.get_addr(1);
    let n = args.get_raw(2);

    file.write(addr, n)
}
```

### close

```rust
pub fn sys_close(args: &SyscallArgs) -> Result<(), SysError> {
    let fd = args.get_raw(0) as usize;
    let mut file = FILE_TABLE.get(fd);
    file.close();
    Ok(())
}
```

## 管道系統呼叫

```rust
pub fn sys_pipe(args: &SyscallArgs) -> Result<(), SysError> {
    let fdarray = args.get_addr(0);
    let (r, w) = log!(Pipe::alloc())?;

    let mut buf = [0u32; 2];
    buf[0] = r.id as u32;
    buf[1] = w.id as u32;

    let proc = current_proc();
    proc::copy_to_user(&buf, fdarray)?;

    Ok(())
}
```

## 信號系統呼叫

```rust
pub fn sys_sigaction(args: &SyscallArgs) -> Result<(), SysError> {
    let sig = args.get_raw(0) as usize;
    let act = args.get_addr(1);
    let old = args.get_addr(2);

    // ...
    Ok(())
}

pub fn sys_sigprocmask(args: &SyscallArgs) -> Result<(), SysError> {
    let how = args.get_raw(0);
    let mask = args.get_raw(1);

    // ...
    Ok(())
}
```

## 記憶體相關系統呼叫

### sbrk

```rust
pub fn sys_sbrk(args: &SyscallArgs) -> Result<usize, SysError> {
    let n = args.get_int(0) as isize;
    let result = unsafe { proc::grow(n, true)? };
    Ok(result)
}
```

### mmap

```rust
pub fn sys_mmap(args: &SyscallArgs) -> Result<VA, SysError> {
    let addr = args.get_raw(0);
    let len = args.get_raw(1);
    let prot = args.get_raw(2);
    let flags = args.get_raw(3);
    let fd = args.get_raw(4);
    let offset = args.get_raw(5);

    // 配置新 VMA
    // ...
}
```

## 時間相關系統呼叫

### sleep

```rust
pub fn sys_sleep(args: &SyscallArgs) -> Result<(), SysError> {
    let ticks = args.get_raw(0);

    let until = TICKS.lock().wrapping_add(ticks);
    while TICKS.lock().0 < until {
        if proc::current_proc().is_killed() {
            err!(SysError::Interrupted);
        }
        proc::sleep(Channel::Ticks, TICKS.lock());
    }

    Ok(())
}
```

## 網路相關系統呼叫

```rust
pub fn sys_socket(args: &SyscallArgs) -> Result<usize, SysError> {
    let domain = args.get_raw(0);
    let stype = args.get_raw(1);
    let protocol = args.get_raw(2);

    net::udp::SocketTable::open(domain, stype, protocol)
}

pub fn sys_send(args: &SyscallArgs) -> Result<usize, SysError> {
    let socket_id = args.get_raw(0);
    let dest_ip = Ipv4Addr::from(args.get_raw(1));
    let dest_port = args.get_raw(2) as u16;
    let buf = args.get_addr(3);
    let len = args.get_raw(4);

    // ...
}

pub fn sys_tcp_socket(args: &SyscallArgs) -> Result<usize, SysError> {
    net::tcp::TcpTable::socket()
}

pub fn sys_tcp_connect(args: &SyscallArgs) -> Result<(), SysError> {
    let id = args.get_raw(0);
    let ip = Ipv4Addr::from(args.get_raw(1));
    let port = args.get_raw(2) as u16;

    net::tcp::TcpTable::connect(id, ip, port)
}
```

## 相關主題

- [[Process]]：程序管理
- [[Trap]]：陷阱處理
- [[file]]：檔案抽象
- [[fs]]：檔案系統