# 管道 — pipe.rs

管道是 Unix 程序間通訊的基礎機制，提供單向位元組流。

## 管道設計

```
程序 A                    程序 B
  │                         ▲
  │ write()                 │ read()
  ▼                         │
┌────────────────────┐     │
│      Pipe          │     │
│  ┌──────────────┐  │     │
│  │  data[512]   │  │     │
│  └──────────────┘  │     │
│  num_read          │     │
│  num_write         │     │
│  read_open         │     │
│  write_open        │     │
└────────────────────┘     │
  │                         │
  └─────────────────────────┘
```

## 資料結構

```rust
const PIPESIZE: usize = 512;

struct PipeInner {
    data: [u8; PIPESIZE],
    num_read: usize,    // 已讀取的位元組數
    num_write: usize,   // 已寫入的位元組數
    read_open: bool,    // 讀端是否開啟
    write_open: bool,   // 寫端是否開啟
}

struct Pipe {
    inner: SpinLock<PipeInner>,
}
```

## 環形緩衝區

使用模運算實現環形緩衝區：

```rust
// 寫入位置
let index = inner.num_write % PIPESIZE;
inner.data[index] = byte;
inner.num_write += 1;

// 讀取位置
let index = inner.num_read % PIPESIZE;
let byte = inner.data[index];
inner.num_read += 1;
```

## 配置管道

```rust
pub fn alloc() -> Result<(File, File), FsError> {
    let pipe = Arc::new(Pipe {
        inner: SpinLock::new(
            PipeInner {
                data: [0; PIPESIZE],
                num_read: 0,
                num_write: 0,
                read_open: true,
                write_open: true,
            },
            "pipe",
        ),
    })?;

    // 配置兩個檔案描述符
    let mut f0 = File::alloc()?;
    let mut f1 = File::alloc()?;

    // f0 = 讀端
    FILE_TABLE.inner[f0.id].lock().type = FileType::Pipe {
        pipe: Arc::clone(&pipe),
    };
    FILE_TABLE.inner[f0.id].lock().readable = true;
    FILE_TABLE.inner[f0.id].lock().writeable = false;

    // f1 = 寫端
    FILE_TABLE.inner[f1.id].lock().type = FileType::Pipe { pipe };
    FILE_TABLE.inner[f1.id].lock().readable = false;
    FILE_TABLE.inner[f1.id].lock().writeable = true;

    Ok((f0, f1))
}
```

## 寫入管道

```rust
pub fn write(&self, addr: VA, n: usize) -> Result<usize, SysError> {
    let (proc, data) = current_proc_and_data_mut();
    let mut inner = self.inner.lock();

    let mut i = 0;
    while i < n {
        if proc.is_killed() {
            err!(SysError::Interrupted);
        }
        if !inner.read_open {
            err!(SysError::BrokenPipe);  // 讀端已關閉
        }

        if inner.num_write == inner.num_read + PIPESIZE {
            // 緩衝區滿，等待消費者讀取
            proc::wakeup(Channel::PipeRead(self.pipe_id()));
            inner = proc::sleep(Channel::PipeWrite(self.pipe_id()), inner);
        } else {
            // 從使用者空間複製一個位元組
            let mut ch = [0u8];
            data.pagetable_mut().copy_from(addr + i, &mut ch)?;

            let index = inner.num_write % PIPESIZE;
            inner.data[index] = ch[0];
            inner.num_write += 1;
            i += 1;
        }
    }

    // 喚醒讀者
    proc::wakeup(Channel::PipeRead(self.pipe_id()));

    Ok(i)
}
```

## 讀取管道

```rust
pub fn read(&self, addr: VA, n: usize) -> Result<usize, SysError> {
    let (proc, data) = current_proc_and_data_mut();
    let mut inner = self.inner.lock();

    // 等待直到有資料或寫端關閉
    while inner.num_read == inner.num_write && inner.write_open {
        if proc.is_killed() {
            err!(SysError::Interrupted);
        }
        inner = proc::sleep(Channel::PipeRead(self.pipe_id()), inner);
    }

    // 管道為空
    if inner.num_read == inner.num_write {
        return Ok(0);
    }

    let mut i = 0;
    while i < n && inner.num_read < inner.num_write {
        let ch = inner.data[inner.num_read % PIPESIZE];
        data.pagetable_mut().copy_to(&[ch], addr + i)?;

        inner.num_read += 1;
        i += 1;
    }

    // 喚醒寫者
    proc::wakeup(Channel::PipeWrite(self.pipe_id()));

    Ok(i)
}
```

## 關閉管道

```rust
pub fn close(&self, writeable: bool) {
    let mut inner = self.inner.lock();

    if writeable {
        inner.write_open = false;
        proc::wakeup(Channel::PipeRead(self.pipe_id()));  // 喚醒讀者
    } else {
        inner.read_open = false;
        proc::wakeup(Channel::PipeWrite(self.pipe_id()));  // 喚醒寫者
    }

    // 當兩端都關閉時，Arc 會自動釋放 Pipe
}
```

## 管道容量

```
PIPESIZE = 512 bytes

讀者                           寫者
   │                              │
   │◄───── num_write - num_read ──►│
   │                              │
   └────── 剩餘空間 ───────────────┘
           PIPESIZE - (num_write - num_read)
```

## 阻塞 vs 非阻塞

- 預設是阻塞的：讀取時等待資料，寫入時等待空間
- 管道空/滿時程序進入睡眠
- 使用 `O_NONBLOCK` 可改為非阻塞（xv8 目前未完整實現）

## Broken Pipe 信號

當寫入已關閉的管道時：
1. 系統呼叫傳回 `SIGPIPE` 錯誤
2. 如果寫入時收到 `EPIPE`，通常表示讀者已終止

## 相關主題

- [[file]]：檔案抽象
- [[fork]]：程序建立
- [[Process]]：程序睡眠/喚醒