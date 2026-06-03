# Process（程序）

程序是 xv8 作業系統中資源分配與執行的基本單位。每個程序都有自己的位址空間、執行緒、控制區塊，並透過系統呼叫與核心互動。xv8 支援最多 64 個並發程序。

## 程序結構

每個程序在核心中由 `proc.rs` 定義的 `Proc` 結構體表示：

```rust
pub struct Proc {
    pub pid: usize,              // 程序 ID
    pub state: ProcState,        // 程序狀態
    pub parent: Option<&'static Proc>,  // 父程序指標
    pub context: Context,        // 上下文（用於上下文切換）
    pub trapframe: &'static mut Trapframe,  // trap 框架
    pub data: *mut u8,           // 使用者資料頁指標
    pub kstack: &'static mut [u8],  // 核心堆疊
    pub name: [u8; 32],         // 程序名稱
    pub fd_table: [Option<Arc<dyn File>>; 16],  // 檔案描述符表
    pub cwd: Arc<dyn fs::INode>, // 目前工作目錄
    pub memory: VmArea,          // 記憶體映射
}
```

## 程序狀態

程序的生命週期包含多種狀態，定義在 `ProcState` 列舉：

- **UNUSED**：程序槽位空閒，尚未使用
- **USED**：已分配但尚未進入可執行狀態
- **SLEEPING**：程序正在等待某事件（如 I/O 完成）
- **RUNNABLE**：程序已就緒，等待 CPU 分配
- **RUNNING**：程序正在 CPU 上執行
- **ZOMBIE**：程序已終止但父程序尚未回收

狀態轉換如下：
```
UNUSED → USED → SLEEPING
               ↓
          RUNNABLE → RUNNING → ZOMBIE
               ↑          ↓
               └──────────┘ (wakeup)
```

## 程序建立：Fork

`fork` 是 Unix 程序建立的核心機制。xv8 實現 Copy-on-Write（COW）fork：

1. 父程序呼叫 `fork` 系統呼叫
2. 核心分配一個新的 `Proc` 結構與新的頁表
3. 父子程序共享所有物理頁，但將頁面標記為唯讀
4. 當任一程序嘗試寫入頁面時，發生頁錯誤，核心才真正複製該頁
5. 子程序獲得自己的頁表副本和 trap 框架
6. 父子程序從 fork 返回時都繼續執行，子程序返回 0，父程序返回子程序的 PID

COW fork 大幅減少記憶體複製成本，因為多數記憶體在 fork 後不會立即被寫入。

## 程序排程

xv8 使用輪詢（Round-Robin）排程器。排程發生在：
- 當前程序放棄 CPU（sleep、wait、exit）
- 時鐘中斷觸發（時間配額用盡）
- 任何可運行程序被 wakeup 喚醒

排程器遍歷程序表，找到第一個處於 RUNNABLE 狀態的程序，透過上下文切換切換到該程序。見 [[Scheduler]]。

## 上下文切換

上下文切換是從一個程序切換到另一個程序的過程。xv8 在 `swtch.rs` 中實現：

```rust
pub fn swtch(from: &mut Context, to: &Context) {
    unsafe {
        core::arch::asm!(
            "sd ra, 0(a0)",
            "sd sp, 8(a0)",
            // ... 保存其他暫存器
            "ld ra, 0(a1)",
            "ld sp, 8(a1)",
            // ... 恢復其他暫存器
        );
    }
}
```

`swtch` 保存當前程序的暫存器到其 `Context`，並從新程序的 `Context` 恢復暫存器。新程序從上次離開的地方繼續執行。

## Sleep 與 Wakeup

程序同步的核心機制。當程序需要等待某事件時：

1. 呼叫 `sleep(channel, lock)`，帶上等待的條件和要釋放的鎖
2. 將自己狀態設為 SLEEPING 並呼叫 `sched()`
3. `sched` 呼叫 `swtch` 切換到排程器
4. 當事件發生時，另一個程序呼叫 `wakeup(channel)`
5. `wakeup` 遍歷所有程序，找到處於 SLEEPING 且等待相同 channel 的程序
6. 將該程序狀態改為 RUNNABLE

## 程式執行：Exec

`exec` 系統呼叫用新程式替換程序的位址空間：

1. 讀取 ELF 格式的可執行檔
2. 驗證檔案格式與權限
3. 為新程式分配新的頁表
4. 載入每個程式區段（text、data、bss）到記憶體
5. 設定新的堆疊
6. 替換 trapframe 中的 PC 值
7. 清空程序的檔案描述符表（除非設定 close-on-exec 標誌）

執行成功後，程序繼續執行新程式的 main 函式，原來的程式碼和資料完全被替換。

## 程式終止：Exit 與 Wait

- `exit()` 將程序狀態設為 ZOMBIE，釋放大部分資源，但保留 Proc 結構供父程序查詢
- `wait()` 父程序呼叫以等待子程序結束。它遍歷程序表，找到處於 ZOMBIE 狀態的子程序，回收其資源並回傳 PID
- 如果父程序先於子程序終止，子程序會被 init 程序（pid=1）收養

## 信號機制

xv8 支援 POSIX 信號機制（見 `signal.rs`）。信號是一種軟體中斷，用於通知程序發生了某事件：

- 標準信號：SIGHUP、SIGINT、SIGQUIT、SIGKILL、SIGTERM、SIGCONT 等
- 程序可以透過 `sigaction` 安裝自訂處理常式
- 預設動作包括終止程序、終止並傾印核心、忽略等

信號處理涉及修改 trapframe 中的 PC，以便從信號處理常式返回後能恢復執行。

## 程序群組與工作階段

xv8 支援程序群組（process groups）和工作階段（sessions）：

- 每個程序屬於一個程序群組（pgid）
- 每個程序群組屬於一個工作階段（sid）
- 終端機產生的信號（Ctrl+C）會傳送給前景程序群組的所有程序
- 管道 `cmd1 | cmd2` 將 cmd1 和 cmd2 放入同一程序群組

## 相關主題

- [[Syscall]]：程式如何透過系統呼叫與核心互動
- [[Scheduler]]：排程器演算法與實現
- [[Virtual-Memory]]：程序的位址空間管理
- [[Trap]]：Trap 處理與信號傳遞
- [[File-System]]：程序的檔案描述符管理