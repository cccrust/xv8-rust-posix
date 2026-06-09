# cgroup 模組 — cgroup.rs

## 理論背景

cgroup (control group, 控制群組) 是核心層級的資源管理機制，用於限制、記錄、隔離 process 群組對 CPU、記憶體、I/O 等資源的使用。xv8 從 v5.2 開始支援 cgroup v2。

cgroup 由 Google 工程師 Paul Menage 和 Rohit Seth 於 2006 年提出，最初稱為 "process containers"。2008 年合併入 Linux 2.6.24，2014 年 Linux 3.16 引入 v2 版本，簡化了層級結構與控制器模型。

cgroup v2 的核心改進：
- 統一層級結構 (unified hierarchy)
- 取消 task 與 cgroup 的雙向關聯
- 執行緒模式 (thread mode) 支援
- 更嚴格的資源控制模型

## xv8 實作

### 字元裝置介面

xv8 使用 `/dev/cgroup` (major=2) 字元裝置作為 cgroup 控制介面，避免複雜的虛擬檔案系統實作。核心解析從裝置讀取的文字命令：

```
create <name>              # 建立 cgroup
attach <pid> <name>        # 將 process 加入 cgroup
delete <name>              # 刪除 cgroup
set <name> cpu.max <v>     # 設定 CPU 限制
set <name> memory.max <v>  # 設定記憶體限制
set <name> pids.max <v>    # 設定 PIDs 限制
```

### 資源限制

| 控制器 | 屬性 | 上限類型 |
|--------|------|---------|
| CPU | `cpu.max` | 配額/週期 (usec) |
| 記憶體 | `memory.max` | 位元組 |
| PIDs | `pids.max` | process 數量 |

### 三個控制器

xv8 的 cgroup 實作三個基本控制器，每個控制器的限制值儲存在 `CgroupNode` 結構中：

```rust
struct CgroupNode {
    name: String,
    children: Vec<CgroupNode>,
    cpu_max: u64,
    memory_max: u64,
    pids_max: u64,
    pids_current: u64,
}
```

### 與 namespace 的整合

cgroup namespace (`CLONE_NEWCGROUP`) 隔離 process 對 cgroup 樹的視圖。結合 `/dev/cgroup` 裝置，每個容器只能看見與自己相關的 cgroup 節點。

## 相關文件

- [Wiki: cgroup](../../../_wiki/cgroup.md)
- [Wiki: 容器](../../../_wiki/Container.md)
- [裝置驅動文件](console.md)
