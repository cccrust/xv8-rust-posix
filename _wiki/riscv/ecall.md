# ecall — 系統呼叫觸發

`ecall` 是 RISC-V 上觸發系統呼叫的指令。

## 機制

```
使用者程式
    │
    │ ecall  // 觸發 exception
    ▼
儲存 sepc = PC
切換到 S-mode
跳到 stvec 指向的 handler
    │
    ▼
scause = EnvironmentCall (8)
    │
    ▼
核心處理 syscall()
    │
    │ 處理完成
    ▼
sret  // 返回使用者模式
```

## 呼叫慣例

```rust
// a7 = 系統呼叫號碼
// a0-a5 = 參數
// 回傳值在 a0
asm!(
    "ecall",
    in("a7") syscall_num,  // 系統呼叫號碼
    in("a0") fd,           // 第一個參數
    in("a1") buf,          // 第二個參數
    in("a2") n,            // 第三個參數
    lateout("a0") ret      // 回傳值
);
```

## 常見系統呼叫

| 號碼 | 名稱 | 目的 |
|------|------|------|
| 1 | fork | 建立程序 |
| 2 | exit | 終止程序 |
| 3 | wait | 等待子程序 |
| 4 | pipe | 建立管道 |
| 5 | read | 讀取 |
| 6 | kill | 傳送信號 |
| 7 | exec | 執行新程式 |
| 9 | open | 開啟檔案 |
| 10 | write | 寫入 |
| 12 | link | 建立連結 |
| 13 | mkdir | 建立目錄 |
| 14 | unlink | 刪除檔案 |
| 15 | close | 關閉檔案描述符 |
| 16 | sleep | 睡眠 |
| 18 | stat | 取得檔案資訊 |
| 19 | chdir | 改變目錄 |
| 20 | getpid | 程序 ID |
| 22 | sbrk | 調整堆積 |
| 24 | getcwd | 目前目錄 |
| 25 | dup | 複製 fd |
| 37 | exit_group | 退出程序 |
| 38 | setpgid | 設定程序群組 |
| 40 | mount | 掛載檔案系統 |

## 使用者空間包裝

```rust
pub fn read(fd: i32, buf: *mut u8, n: usize) -> i32 {
    let mut ret: i32;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") 4,              // read syscall number
            in("a0") fd,
            in("a1") buf as usize,
            in("a2") n,
            lateout("a0") ret
        );
    }
    ret
}
```

## 參數傳遞

最多 6 個參數通過 a0-a5 傳遞：

```rust
// openat(path, flags, mode)
asm!(
    "ecall",
    in("a7") 56,              // openat
    in("a0") dirfd,
    in("a1") path.as_ptr(),
    in("a2") flags,
    in("a3") mode,
    lateout("a0") ret
);
```

## 回傳值

```rust
// 成功：非負值
// 失敗：-1，並設定 errno
if ret < 0 {
    errno = -ret;
    ret = -1;
}
```

## 核心處理

```rust
pub fn syscall(num: usize, args: &[usize; 6]) -> isize {
    match num {
        1 => sys_clone(args[0]),
        2 => sys_exit(args[0] as i32),
        3 => sys_wait(args[0] as *mut i32),
        4 => sys_pipe(args[0] as *mut i32),
        5 => sys_read(args[0] as i32, args[1] as *mut u8, args[2]),
        6 => sys_kill(args[0] as i32, args[1] as i32),
        7 => sys_exec(args[0] as *const u8),
        9 => sys_open(args[0] as *const u8, args[1] as i32),
        10 => sys_write(args[0] as i32, args[1] as *const u8, args[2]),
        // ...
        _ => -1,
    }
}
```

## sepc 和回返回位置

`sepc` 儲存 ecall 指令的位址，`sret` 會回到 `sepc + 4`（跳過 ecall 指令）。

```rust
// 取得使用者 PC
let epc = sepc::read();
// sepc 指向 ecall 指令

// 返回時會跳到 ecall 的下一條指令
// （在某些情況下可能需要修改 sepc 指向其他位置）
```

## 與 x86 syscall 的比較

| 特性 | RISC-V ecall | x86 syscall |
|------|--------------|-------------|
| 觸發方式 | exception | 快速系統呼叫 |
| 模式切換 | 必須 | 必須 |
| 暫存器 | a7 = 號碼 | rax = 號碼 |
| 參數 | a0-a5 | rdi, rsi, rdx, r10, r8, r9 |
| 返回 | a0 | rax |

## 多系統呼叫

xv8 使用連續的 ecall 進行多個系統呼叫，例如：

```rust
write(fd, buf, n);  // ecall
read(fd, buf, n);   // ecall
```

每次 ecall 都會：
1. 觸發 trap
2. 進入核心
3. 處理系統呼叫
4. 返回

## 效能考量

- 每個 ecall 都有模式切換開銷
- Linux 有 vDSO 避免某些系統呼叫
- xv8 目前無此優化

## 錯誤處理

```rust
if let Err(e) = do_syscall() {
    // 設定 errno
    errno = e.code();
    // 返回 -1
    return -1;
}
```

## 阻塞系統呼叫

某些系統呼叫可能阻塞（例如 read 等待輸入）：

```rust
// 在核心中
pub fn sys_read(...) -> isize {
    // 如果無資料可用
    sleep(&chan);  // 讓出 CPU
    // 醒來後繼續
}
```

## 安全性

- 核心檢查所有指標參數
- 使用者無法偽造系統呼叫號碼
- 記憶體訪問受到 PMP 和分頁限制