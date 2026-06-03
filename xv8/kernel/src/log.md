# 預寫式日誌 — log.rs

預寫式日誌（Write-Ahead Logging, WAL）確保檔案系統在崩潰後能恢復到一致狀態。

## 核心思想

```
1. 修改先寫入日誌區
2. 日誌提交後，將修改應用到實際位置
3. 如果崩潰發生在提交前，日誌可用於重做或撤銷
```

## 磁碟佈局

```
┌──────────┬────────────────────┬────────────────────┐
│ Header   │  Log Block 1      │  Log Block 2 ...  │
│ (1 blk)  │  (modifed data)   │                   │
└──────────┴────────────────────┴────────────────────┘
     │            │                    │
     ↓            ↓                    ↓
 日誌開始      資料備份               更多資料
```

## 日誌格式

```rust
struct LogHeader {
    n: u32,               // 已記錄的區塊數量
    blocks: [u32; 30],    // 區塊編號列表
}
```

## 交易機制

```rust
struct LogInner {
    outstanding: u32,     // 正在進行的 FS 系統呼叫數
    committing: bool,      // 是否正在提交
    header: LogHeader,
    // ...
}

// 使用 Operation 確保 begin/end 配對
pub struct Operation<F: FnOnce() = fn()> {
    on_err: Option<F>,
    success: bool,
}

impl Operation {
    pub fn begin() -> Self {
        begin_op();  // 增加 outstanding
        Self { on_err: None, success: false }
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        if !self.success && let Some(f) = self.on_err.take() {
            f();  // 錯誤時回呼
        }
        end_op();  // 減少 outstanding，可能觸發提交
    }
}
```

## 開始操作

```rust
fn begin_op() {
    let mut inner = LOG.inner.lock();

    loop {
        if inner.committing {
            // 等待提交完成
            inner = proc::sleep(Channel::Log, inner);
        } else if inner.header.n + (inner.outstanding + 1) * MAXOPBLOCKS > LOGBLOCKS {
            // 日誌空間可能不足，等待
            inner = proc::sleep(Channel::Log, inner);
        } else {
            inner.outstanding += 1;
            break;
        }
    }
}
```

## 寫入記錄

```rust
pub fn write(buf: &Buf<'_>) {
    let mut inner = LOG.inner.lock();

    if inner.header.n >= LOGBLOCKS || inner.header.n >= inner.size - 1 {
        panic!("transaction too big");
    }

    if inner.outstanding < 1 {
        panic!("log_write outside of transaction");
    }

    let block_no = BCACHE.inner.lock().meta[buf.id].block_no;

    // 日誌吸收：如果同一區塊已被記錄，跳過
    let mut i = 0;
    while i < inner.header.n {
        if inner.header.blocks[i] == block_no {
            break;
        }
        i += 1;
    }

    inner.header.blocks[i] = block_no;

    if i == inner.header.n as usize {
        BCACHE.pin(buf);  // 固定防止回收
        inner.header.n += 1;
    }
}
```

## 提交交易

```rust
fn end_op() {
    let mut do_commit = false;

    {
        let mut inner = LOG.inner.lock();
        inner.outstanding -= 1;

        if inner.committing {
            panic!("log committing");
        }

        if inner.outstanding == 0 {
            do_commit = true;
            inner.committing = true;
        } else {
            proc::wakeup(Channel::Log);
        }
    }

    if do_commit {
        commit();
        let mut inner = LOG.inner.lock();
        inner.committing = false;
        proc::wakeup(Channel::Log);
    }
}

fn commit() {
    let n = LOG.inner.lock().header.n;

    if n > 0 {
        // 1. 將修改的區塊寫入日誌
        write_log();
        // 2. 寫入日誌標頭（真正的提交點）
        unsafe { write_head() };
        // 3. 將區塊從日誌安裝到實際位置
        install_trans(false);
        // 4. 清除日誌
        {
            let mut inner = LOG.inner.lock();
            inner.header.n = 0;
        }
        unsafe { write_head() };
    }
}
```

## 安裝交易

```rust
fn install_trans(recovering: bool) {
    for tail in 0..n {
        let block = LOG.inner.lock().header.blocks[tail];

        // 從日誌讀取
        let lbuf = BCACHE.read(dev, start + tail + 1);
        // 讀取目標位置
        let mut dbuf = BCACHE.read(dev, block);

        // 複製資料
        dbuf.data_mut().copy_from_slice(lbuf.data());

        // 寫入磁碟
        BCACHE.write(&mut dbuf);

        if !recovering {
            BCACHE.unpin(&dbuf);
        }

        BCACHE.release(lbuf);
        BCACHE.release(dbuf);
    }
}
```

## 崩潰恢復

```rust
pub unsafe fn recover_from_log() {
    // 1. 讀取日誌標頭
    Log::read_head();

    // 2. 如果有已提交的記錄，安裝到實際位置
    Log::install_trans(true);

    // 3. 清除日誌
    {
        let mut inner = LOG.inner.lock();
        inner.header.n = 0;
    }
    unsafe { Log::write_head() };
}
```

## 恢復流程

```
開機
    ↓
讀取超級區塊
    ↓
初始化日誌系統
    ↓
recover_from_log()
    ↓
有已提交交易？──是──→ 將區塊從日誌複製到實際位置
    ↓                  ↓
清除日誌            繼續開機
    ↓
正常操作
```

## 日誌吸收

當同一交易中多次修改同一區塊時，後續修改被吸收（不重複記錄）：

```rust
let mut i = 0;
while i < inner.header.n {
    if inner.header.blocks[i] == block_no {
        break;  // 已存在，跳過
    }
    i += 1;
}
inner.header.blocks[i] = block_no;
if i == inner.header.n as usize {
    BCACHE.pin(buf);
    inner.header.n += 1;
}
```

## 限制

- 日誌大小有限（30 個區塊）
- 每個交易修改的區塊數受限於 `MAXOPBLOCKS * 3`
- 交易期間不能釋放鎖

## 相關主題

- [[fs]]：檔案系統
- [[buf]]：緩衝區塊快取
- [[virtio_disk]]：磁碟驅動