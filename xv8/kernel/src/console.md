# 主控台 — console.rs

主控台是使用者輸入輸出的核心介面，連接到 UART 序列埠。

## 輸入環形緩衝區

```rust
pub struct Console {
    buf: [u8; 128],    // 環形緩衝區
    r: usize,          // 讀取索引
    w: usize,          // 已完成輸入索引
    e: usize,          // 編輯位置索引
    raw: bool,         // Raw 模式
    foreground_pid: Option<Pid>,  // 前台程序
}
```

緩衝區狀態：
```
buf: [ 已消費 |    已完成（行）    | 正在編輯 ]
      ───────^r───────────────^w─────────^e
```

## Raw 模式與 Cooked 模式

**Cooked 模式**：
- 行緩衝
- Ctrl-C 殺死前台程序
- Ctrl-U 殺死行
- Ctrl-D 發送 EOF
- 收到 `\n` 後才返回給讀者

**Raw 模式**：
- 字元導向
- 每個字元立即返回
- 無行編輯

## 讀取輸入

```rust
pub fn read(&self, dst: VA, mut len: usize) -> Result<usize, SysError> {
    let mut console = CONSOLE.lock();

    while console.r == console.w {
        if proc::current_proc().is_killed() {
            err!(SysError::Interrupted);
        }
        // 睡眠等待輸入
        console = proc::sleep(Channel::Buffer(&console.r as *const _ as usize), console);
    }

    while len > 0 {
        let c = console.buf[console.r % INPUT_BUF_SIZE];
        console.r += 1;

        // 處理 Ctrl-D EOF
        if !console.raw && c == ctrl(b'D') {
            if len < target {
                console.r -= 1;
            }
            break;
        }

        // 複製到使用者空間
        proc::copy_to_user(&[c], dst)?;

        dst += 1;
        len -= 1;

        // Raw 模式或收到換行符
        if c == b'\n' || console.raw {
            break;
        }
    }

    Ok(target - len)
}
```

## 中斷處理

```rust
pub fn handle_interrupt(c: u8) {
    let mut console = CONSOLE.lock();

    if console.raw {
        // Raw 模式：直接儲存並喚醒讀者
        console.buf[console.e % INPUT_BUF_SIZE] = c;
        console.e += 1;
        console.w = console.e;
        proc::wakeup(Channel::Buffer(&console.r as *const _ as usize));
        return;
    }

    match c {
        // Backspace
        c if c == ctrl(b'H') || c == b'\x7f' => {
            if console.e != console.w {
                console.e -= 1;
                Console::put_backspace();
            }
        }

        // Ctrl-U: 殺死行
        c if c == ctrl(b'U') => {
            while console.e != console.w {
                console.e -= 1;
                Console::put_backspace();
            }
        }

        // Ctrl-C: 殺死程序
        c if c == ctrl(b'C') => {
            if let Some(pid) = console.foreground_pid {
                proc::kill(pid);
            }
        }

        // 普通字元
        _ => {
            if c != 0 && console.e - console.r < INPUT_BUF_SIZE {
                if c == b'\r' {
                    c = b'\n';  // 轉換 CR 到 LF
                }

                Self::putc_sync(c);  // 回顯

                console.buf[console.e % INPUT_BUF_SIZE] = c;
                console.e += 1;

                // 行完成或緩衝區滿
                if c == b'\n' || c == ctrl(b'D') || console.e - console.r == INPUT_BUF_SIZE {
                    console.w = console.e;
                    proc::wakeup(Channel::Buffer(&console.r as *const _ as usize));
                }
            }
        }
    }
}
```

## 寫出輸出

```rust
pub fn write(&self, src: VA, len: usize) -> Result<usize, SysError> {
    let mut n = 0;
    let mut buf = [0u8; 32];

    let raw = CONSOLE.lock().raw;

    while n < len {
        let chunk = 32.min(len - n);
        proc::copy_from_user(src, &mut buf[..chunk])?;

        if raw {
            uart::write_sync(&buf[..chunk]);  // Raw 模式同步寫
        } else {
            uart::write(&buf[..chunk]);        // Cooked 模式中斷驅動
        }
        n += chunk;
        src += chunk;
    }

    Ok(len)
}
```

## Ioctl 控制

```rust
pub fn ioctl(cmd: usize, arg: usize) -> Result<usize, SysError> {
    match cmd {
        Ioctl::CONSOLE_SET_RAW => {
            if arg == 1 {
                console.raw = true;
                console.e = console.w;
            } else {
                console.raw = false;
            }
            Ok(0)
        }
        Ioctl::CONSOLE_SET_FG_PID => {
            console.foreground_pid = if arg == 0 {
                None
            } else {
                Some(unsafe { Pid::from_usize(arg) })
            };
            Ok(0)
        }
        _ => Err(SysError::InvalidArgument),
    }
}
```

## Ctrl 組合鍵

```rust
const fn ctrl(c: u8) -> u8 {
    c.wrapping_sub(b'@')
}

ctrl(b'C') = 0x03  // ETX (End of Text)
ctrl(b'D') = 0x04  // EOT (End of Transmission)
ctrl(b'U') = 0x15  // NAK (Negative Ack)
ctrl(b'H') = 0x08  // BS (Backspace)
```

## 特殊組合鍵

| 按鍵 | ASCII | 動作 |
|------|-------|------|
| Ctrl-C | 0x03 | 殺死前台程序 |
| Ctrl-D | 0x04 | EOF（Cooked 模式） |
| Ctrl-U | 0x15 | 刪除行 |
| Ctrl-H | 0x08 | 退格 |
| DEL | 0x7F | 退格 |

## 相關主題

- [[uart]]：序列埠硬體
- [[trap]]：中斷處理
- [[file]]：檔案抽象