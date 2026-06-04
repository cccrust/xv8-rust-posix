# ps — 程序狀態

`ps`（process status）顯示目前系統中程序的快照。

## 核心設計

```rust
if all {
    #[cfg(unix)]
    {
        let proc_dir = Path::new("/proc");
        if proc_dir.is_dir() {
            for entry in std::fs::read_dir(proc_dir).unwrap() {
                if let Ok(pid) = name.to_string_lossy().parse::<u32>() {
                    let stat_path = proc_dir.join(&name).join("stat");
                    if let Ok(content) = std::fs::read_to_string(&stat_path) {
                        let fields: Vec<&str> = content.splitn(4, ' ').collect();
                        if fields.len() >= 2 {
                            let comm = fields[1].trim_matches('(').trim_matches(')');
                            println!("{:>5} {}", pid, comm);
                        }
                    }
                }
            }
        }
    }
} else {
    // 預設：只顯示目前程序
    let pid = std::process::id();
    let name = std::env::args().next().unwrap_or_else(|| "?".to_string());
    println!("{:>5} {}", pid, name);
}
```

`ps` 透過讀取 `/proc` 檔案系統來獲取程序資訊。

## /proc 檔案系統

Linux 的 `/proc` 是一個虛擬檔案系統，提供程序和系統資訊：

- `/proc/{pid}/stat`：程序狀態
- `/proc/{pid}/cmdline`：命令列
- `/proc/{pid}/status`：詳細狀態
- `/proc/{pid}/fd/`：開啟的檔案描述符

## 解析 /proc/{pid}/stat

```rust
if let Ok(content) = std::fs::read_to_string(&stat_path) {
    let fields: Vec<&str> = content.splitn(4, ' ').collect();
    // fields[0]: pid
    // fields[1]: command name in parentheses
    let comm = fields[1].trim_matches('(').trim_matches(')');
    println!("{:>5} {}", pid, comm);
}
```

`/proc/{pid}/stat` 格式：
```
pid (comm) state ppid pgrp session tty_nr ...
```

命令名在括號中，需要特殊處理。

## 程序狀態

`/proc/{pid}/stat` 中的狀態欄位：
- `R`：執行中（Running）
- `S`：睡眠（Sleeping）
- `D`：磁碟等待（Disk sleep）
- `Z`：殭屍（Zombie）
- `T`：已停止（Stopped）
- `I`：空閒（Idle）

## 選項處理

```rust
let mut all = false;  // -a：顯示所有程序

match c {
    'a' => all = true,
    _ => { eprintln!("ps: invalid option -- '{}'", c); }
}
```

xv8 的 `ps` 簡化了，只支援 `-a` 選項。

## 預設行為

預設模式下，只顯示目前程序：

```bash
ps
# PID COMMAND
# 1234 bash
```

## -a 選項

`-a` 顯示所有程序（遍历所有數字 PID 目錄）：

```bash
ps -a
#  PID COMMAND
#    1 init
#  567 bash
# 1234 ps
```

## 與其他系統的比較

- Linux `ps`：完整功能，讀取 `/proc`
- BSD `ps`：不同選項語法
- macOS `ps`：混合風格

## 完整 ps 的輸出範例

```bash
ps aux
# USER   PID %CPU %MEM   VSZ   RSS TTY   STAT START   TIME COMMAND
# root     1  0.0  0.1   1234   456 ?     S    10:00   0:02 init
# user   567  0.1  0.2   5678  2345 pts/0  S    10:01   0:05 bash
```

欄位說明：
- **USER**：擁有者
- **PID**：程序 ID
- **%CPU**：CPU 使用率
- **%MEM**：記憶體使用率
- **VSZ**：虛擬記憶體大小
- **RSS**：實際記憶體（-resident Set Size）
- **TTY**：控制的終端機
- **STAT**：程序狀態
- **START**：啟動時間
- **TIME**：累計 CPU 時間
- **COMMAND**：命令

## 程序關聯

TTY 欄位顯示程序關聯的終端：
- `?`：無終端（如守護程式）
- `pts/0`：虛擬終端
- `tty1`：系統終端

## 與 top 的比較

| 特性 | `ps` | `top` |
|------|------|-------|
| 更新 | 靜態快照 | 動態更新 |
| 頻率 | 一次性 | 持續運行 |
| 用途 | 瞬間檢視 | 監視負載 |

## 底層系統呼叫

`ps` 主要依賴：
- `readdir`：列舉 `/proc`
- `read`：讀取 `/proc/{pid}/*`

不需要特殊的系統呼叫。

## 安全考量

`ps` 讀取 `/proc` 來獲取資訊，在容器中可能有不同的視圖。

## 實用範例

```bash
# 顯示所有程序
ps aux

# 顯示特定程序
ps aux | grep nginx

# 顯示程序樹
ps axjf

# 只顯示 PID
ps -eo pid,comm
```

## 信號傳遞

```bash
# 殺掉程序
ps aux | grep nginx | awk '{print $2}' | xargs kill
```

結合其他工具傳送信號。

## 相關指令

- `top`：互動式程序監視
- `htop`：增強的 top
- `pstree`：程序樹狀圖
- `pgrep`：搜尋程序