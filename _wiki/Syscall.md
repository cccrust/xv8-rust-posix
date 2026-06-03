# Syscall（系統呼叫）

系統呼叫（syscall）是使用者空間程式與 xv8 核心之間的基本介面。它們提供受控的權限來執行特权操作，如檔案 I/O、程序建立和記憶體配置，這些操作在使用者模式下無法直接執行。

## 系統呼叫介面

在 RISC-V 上，系統呼叫使用 `ecall` 指令。呼叫慣例傳遞：

- `a7`（x17）：系統呼叫號碼，決定要執行哪個操作
- `a0-a5`（x10-x15）：最多六個引數，根據系統呼叫而異
- 回傳值放在 `a0`（x10）：負值通常表示錯誤（與 xv6 一致）

當使用者程式執行 `ecall` 時，硬體從使用者模式轉換到監督者模式，跳到 trap 向量。核心接著：
1. 儲存使用者程式的暫存器
2. 從 `a7` 解碼系統呼叫號碼
3. 分派到對應的處理函式
4. 透過 `sret` 返回使用者模式

## xv8 系統呼叫號碼

系統呼叫號碼定義在 `kernel/src/syscall.rs`。xv8 實作了超過 50 個系統呼叫，包括：

| 號碼 | 名稱 | 用途 |
|------|------|------|
| 1 | fork | 建立新程序（Copy-on-Write） |
| 2 | exit | 終止目前程序 |
| 3 | wait | 等待子程序結束 |
| 4 | pipe | 建立程序間通訊的管道 |
| 5 | read | 從檔案描述符讀取 |
| 6 | kill | 傳送信號給程序 |
| 7 | exec | 用新程式替換目前程序 |
| 9 | open | 開啟或建立檔案 |
| 10 | write | 寫入檔案描述符 |
| 11 | mknod | 建立裝置檔案 |
| 12 | unlink | 刪除檔案 |
| 13 | link | 建立硬連結 |
| 14 | mkdir | 建立目錄 |
| 15 | close | 關閉檔案描述符 |
| 16 | sleep | 休眠指定 ticks |
| 19 | getpid | 取得目前程序 ID |
| 20 | getppid | 取得父程序 ID |
| 22 | sbrk | 調整堆積指標 |
| 24 | getcwd | 取得目前工作目錄 |
| 25 | dup | 複製檔案描述符 |
| 27 | signal | 設定信號處理常式 |
| 29 | sync | 清除檔案系統緩衝區 |
| 32 | getdents | 讀取目錄項目 |
| 35 | sysinfo | 取得系統資訊 |
| 36 | clone | 建立新執行緒/程序 |
| 37 | exit_group | 退出程序中的所有執行緒 |
| 38 | setpgid | 設定程序群組 ID |
| 39 | getpgid | 取得程序群組 ID |
| 40 | getsid | 取得工作階段 ID |
| 41 | mount | 掛載檔案系統 |
| 42 | umount2 | 卸載檔案系統 |

## 使用者空間 API

使用者程式透過 `user/src/syscall.rs` 中的包裝函式與系統呼叫互動。以 `read` 系統呼叫為例：

```rust
pub fn read(fd: i32, buf: *mut u8, n: usize) -> i32 {
    let mut ret: i32;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") 4,        // 系統呼叫號碼
            in("a0") fd,       // 檔案描述符
            in("a1") buf as usize,  // 緩衝區指標
            in("a2") n,        // 位元組數
            lateout("a0") ret
        );
    }
    ret
}
```

每個包裝函式使用內嵌組語將引數載入正確的暫存器，然後執行 `ecall`。`a7` 暫存器保存系統呼叫號碼，核心的 trap 處理器用它來分派。

## 系統呼叫回傳值

xv8 遵循 Unix 慣例：成功的呼叫回傳非負值（讀取/寫入的位元組數、檔案描述符、程序 ID），錯誤時回傳 -1 並設定 `errno`。核心的 `syscall` 函式在錯誤時回傳 `usize::MAX`，然後由使用者函式庫轉換為 -1。

## 參數傳遞

核心透過已儲存的使用者暫存器來存取系統呼叫引數。在 `syscall.rs` 中：

```rust
pub fn syscall(num: usize, args: &[usize; 6]) -> isize {
    match num {
        1 => sys_clone(args[0]),
        2 => sys_exit(args[0] as i32),
        3 => sys_wait(args[0] as *mut i32),
        // ...
    }
}
```

## Trap 路徑

當使用者程式發出 `ecall` 時：

1. 硬體儲存 `sepc`（使用者 PC）、`sstatus` 和 `scause`
2. 將 `stvec` 設為下一個 PC（核心 trap 向量）
3. 跳躍到監督者模式
4. 核心的 `kernelvec`（在 `kernel/src/kernelvec.rs`）將暫存器儲存到使用者 trap 框架
5. `trap.rs` 根據 `a7` 分派到 `syscall()`
6. 處理完成後，`usertrapret()` 準備透過 `sret` 返回

## 相關主題

- [[Process]]：程序如何使用系統呼叫
- [[Trap]]：詳細的 trap 處理流程
- [[File-System]]：檔案相關的系統呼叫
- [[Shell]]：shell 如何使用系統呼叫