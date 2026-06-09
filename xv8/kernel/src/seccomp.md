# seccomp 模組 — seccomp.rs

## 理論背景

seccomp (secure computing mode) 是核心層級的系統呼叫過濾機制，允許 process 限制自身可使用的系統呼叫集合。容器安全的三個核心支柱為 namespace（資源隔離）、capabilities（權限分割）、seccomp（系統呼叫過濾）。

seccomp 的概念最初由 Andrea Arcangeli 在 2005 年提出，Linux 2.6.12 納入核心。原始模式僅允許四種系統呼叫：`read`、`write`、`_exit`、`sigreturn`。2012 年 Linux 3.5 引入 seccomp-bpf，使用 BPF (Berkeley Packet Filter) 位元組碼定義過濾規則。

## xv8 實作

### BPF 過濾器執行引擎

xv8 的 seccomp 實作使用指令指標 (instruction pointer) while 迴圈執行 BPF 過濾器，而非完整的 BPF 虛擬機：

```rust
pub fn seccomp_check(syscall_num: usize) -> bool {
    let proc = current_proc();
    let data = proc.data();
    let filter = match &data.seccomp {
        Some(f) => f,
        None => return true,  // 無過濾器，允許所有
    };
    run_bpf_filter(filter, syscall_num)
}
```

### BPF 虛擬機

BPF 過濾器由 `SockFilter` 指令陣列組成：

| 類別 | 指令 | 說明 |
|------|------|------|
| BPF_LD | `0x00` | 載入資料到累加器 A 或索引暫存器 X |
| BPF_LDX | `0x01` | 載入資料到 X |
| BPF_ST | `0x02` | 儲存 A 到記憶體 |
| BPF_STX | `0x03` | 儲存 X 到記憶體 |
| BPF_ALU | `0x04` | 算術運算 |
| BPF_JMP | `0x05` | 條件跳躍 |
| BPF_RET | `0x06` | 回傳動作值 |
| BPF_MISC | `0x07` | 其他操作 |

### seccomp_check 的特殊處理

系統呼叫 144 (`seccomp` 本身) 始終被跳過 (SKIP)，原因如下：
- 防止 process 設定 seccomp 後因規則錯誤而無法修正
- 讓 init process 可動態管理容器的安全策略

```rust
// seccomp.rs 中的特殊處理
if syscall_num == 144 { return true; }
```

### 資料結構

```rust
pub struct SockFprog {
    pub len: u16,
    pub filter: *const SockFilter,
}

pub struct SockFilter {
    pub code: u16,  // BPF 指令碼
    pub jt: u8,     // 真跳躍
    pub jf: u8,     // 假跳躍
    pub k: u32,     // 通用欄位
}
```

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 144 | `seccomp` | `(operation: u32, flags: u32, filter: *const SockFprog)` |

## 相關文件

- [Wiki: seccomp](../../../_wiki/seccomp.md)
- [Wiki: 容器](../../../_wiki/Container.md)
- [Wiki: Capability](../../../_wiki/Capability.md)
- [syscall 文件](syscall.md)
