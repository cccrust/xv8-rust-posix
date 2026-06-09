# cgroup (控制群組)

## 概述

cgroup (control group, 控制群組) 是 Linux 核心提供的資源管理機制，用於限制、記錄、隔離 process 群組對 CPU、記憶體、I/O 等資源的使用。xv8 從 v5.2 開始支援 cgroup v2。

cgroup 的概念最初由 Google 工程師在 2006 年提出，名為 "process containers"，2008 年合併入 Linux 2.6.24。

## 核心概念

### cgroup v2 層級結構

cgroup 以樹狀結構組織，每個節點代表一個 process 群組：

```
/sys/fs/cgroup/
├── cpu.max
├── memory.max
├── pids.max
├── mycontainer/          # 使用者建立的 cgroup
│   ├── cpu.max
│   ├── memory.max
│   ├── pids.max
│   └── cgroup.procs      # 包含在此 cgroup 的 process 列表
└── ...
```

### xv8 的簡化模型

xv8 使用字元裝置 (character device) `/dev/cgroup` (major=2) 與使用者空間通訊。核心透過文字協定接收命令：

```text
# xv8 cgroup 文字協定
create <name>              # 建立 cgroup
attach <pid> <name>        # 將 process 加入 cgroup
set <name> cpu.max <val>   # 設定 CPU 上限
set <name> memory.max <val> # 設定記憶體上限
set <name> pids.max <val>  # 設定最大 process 數
delete <name>              # 刪除 cgroup
```

## 控制器

### CPU 控制器

限制 process 群組可使用的 CPU 時間。透過 `cpu.max` 檔案設定：

```text
# 格式: <配額> <週期>
# 範例: 50000 100000 → 單核心的 50%
set mycontainer cpu.max 50000 100000
```

### 記憶體控制器

限制 process 群組可使用的實體記憶體上限：

```text
# 格式: <上限位元組>
# 範例: 64MB
set mycontainer memory.max 67108864
```

xv8 的記憶體控制器目前回傳記憶體總量與已用量，以字節為單位：

```text
# 讀取 cgroup stats 輸出範例
mycontainer: memory 67108864/12345678 cpu 50000/100000 pids 5/10
```

### PIDs 控制器

限制 process 群組可建立的 process 總數：

```text
# 格式: <上限>
# 範例: 最多 10 個 process
set mycontainer pids.max 10
```

## xv8 實作細節

### 核心端

`kernel/src/cgroup.rs` 實作 cgroup 的核心邏輯：

1. **cgroup 樹**: 使用 `CgroupNode` 結構體樹狀管理所有 cgroup
2. **文字協定解析器**: 從 `/dev/cgroup` 讀取使用者命令並執行
3. **資源追蹤**: 每個 cgroup 記錄 CPU、記憶體、PIDs 的使用情況

### 字元裝置介面

```rust
// /dev/cgroup 的讀寫入口
pub fn device_read(addr: VA, n: usize) -> Result<usize, SysError>;
pub fn device_write(addr: VA, n: usize) -> Result<usize, SysError>;
```

`/dev/cgroup` 為 major=2 的字元裝置，註冊在 `DEVICES[2]`：

```rust
pub const CGROUP_DEV: usize = 2;
```

### 與 namespace 的互動

cgroup namespace (`CLONE_NEWCGROUP`) 隔離 process 對 cgroup 樹的視圖。當 process 建立新的 cgroup namespace 後，它只能看到自己的 cgroup 及其子節點。

## 使用範例

```rust
// 開啟 cgroup 裝置
let cg = open("cgroup", OpenFlag::READ_WRITE).unwrap();

// 建立 cgroup
write(cg, b"create mycontainer\n").unwrap();

// 將自己加入 cgroup
let pid = getpid();
write(cg, format!("attach {} mycontainer\n", pid).as_bytes()).unwrap();

// 讀取統計資訊
let mut buf = [0u8; 512];
let n = read(cg, &mut buf).unwrap();
```

## 相關文件

- [Wiki: 容器](Container.md)
- [Wiki: Namespace](Namespace.md)
- [Wiki: Capability](Capability.md)
- [xv8 kernel: cgroup.rs](../../xv8/kernel/src/cgroup.md)
- [_doc/v5.2.md](../../_doc/v5.2.md)
