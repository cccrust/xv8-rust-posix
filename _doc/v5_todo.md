
---

## v5.x — Docker / 容器基礎設施

> 在 xv8 中加入類似 Docker 的容器化基礎，提供 namespace 隔離、cgroup 資源限制、seccomp 安全過濾、overlayfs 分層檔案系統、veth 容器網路等。

Docker 在 Linux 上依賴的核心 kernel 功能：

```
┌─────────────────────────────────────────────────┐
│                   Docker CLI/daemon              │
├─────────┬─────────┬──────────┬────────┬─────────┤
│  Linux   │         │          │        │         │
│  Capabi- │ seccomp │  cgroups │  Over- │  Veth   │
│  lities  │  (BPF)  │   v2     │  layFS │ +Bridge │
├─────────┴─────────┴──────────┴────────┴─────────┤
│          Namespaces (7 types)                    │
│  PID │ Mount │ Net │ UTS │ IPC │ User │ Cgroup   │
├──────────────────────────────────────────────────┤
│                  xv8 Kernel                      │
└──────────────────────────────────────────────────┘
```

### 總體里程碑

| 版本 | 主題 | 核心新增 | 依賴 v4.x |
|------|------|---------|-----------|
| v5.1 | Namespaces | `clone(CLONE_NEW*)`, `unshare`, `setns` | memfd, pidfd |
| v5.2 | cgroups v2 | cgroupfs + CPU/Mem/PID controllers | — |
| v5.3 | 安全隔離 | Capabilities + seccomp-BPF | prctl |
| v5.4 | OverlayFS | 堆疊式聯合掛載檔案系統 | memfd, splice |
| v5.5 | 容器網路 | veth pair, bridge, NAT | — |
| v5.6 | 容器執行環境 | `pivot_root`, `sethostname`, container lifecycle | v5.1–v5.5 |
| v5.7 | xv8-container 工具 | Docker-like CLI (pull/run/exec/ps/rm) | v5.6 |

---

## v5.1 — Namespaces (PID / Mount / Network / UTS / IPC / User / Cgroup)

### 目標

實作 Linux 風格的 7 種 namespace，使 process 能擁有隔離的視圖。這是容器化的核心。

### Kernel 新增

#### clone 新 flags

```c
// 新 CLONE_NEW* flags (與 Linux riscv64 共用)
#define CLONE_NEWPID   0x02000000  // PID namespace
#define CLONE_NEWNS    0x00020000  // Mount namespace
#define CLONE_NEWNET   0x40000000  // Network namespace
#define CLONE_NEWUTS   0x04000000  // UTS (hostname) namespace
#define CLONE_NEWIPC   0x08000000  // IPC namespace
#define CLONE_NEWUSER  0x10000000  // User namespace
#define CLONE_NEWCGROUP 0x02000000 // Cgroup namespace
```

| Syscall | 編號 | 功能 |
|---------|------|------|
| `setns` | 140 | `setns(fd, nstype)` — 加入已存在的 namespace |
| `unshare` | 141 | `unshare(flags)` — 將 process 從父的 namespace 分離 |

#### PID namespace 實作重點

```
xv8 現狀                 →  加入 PID namespace 後
init (PID 1)             →  每個 namespace 有自己的 PID 1
kill(pid) 全域尋址       →  pid 在 namespace 內隔離
/proc 顯示全部 process   →  只顯示同 namespace 的 process
```

- `struct Proc` 增加 `ns_pid` 欄位（namespace 內的 PID）
- `getpid()` 回傳 `ns_pid` 而非全域 PID
- `sys_kill()` 查詢 PID 時限於同 namespace
- 每個 namespace 有獨立的 `pid_map`（全域 PID → namespace PID）

#### Mount namespace 實作重點

- 每個 mount namespace 有獨立的 mount table
- `clone(CLONE_NEWNS)` 拷貝父 namespace 的 mount table
- 後續 `mount()`/`umount()` 只影響該 namespace
- 實作 `mount()` syscall（xv8 目前無 mount/unmount）

#### Network namespace 實作重點

- 每個 net namespace 有獨立的網路堆疊
- 獨立的路由表、ARP table、socket 列表
- `clone(CLONE_NEWNET)` 新 namespace 的網路為初始空狀態
- 需與 v5.5 veth pair 搭配才能對外通訊

#### UTS / IPC / User / Cgroup namespace 實作重點

| Type | 隔離內容 | 實作難度 |
|------|---------|---------|
| UTS | `hostname`, `domainname` | 低 — 新增 `sethostname`/`gethostname` syscall |
| IPC | System V IPC (sem, shm, msg) | 中 — xv8 目前無 System V IPC，需從頭加入 |
| User | UID/GID mapping | 高 — 需 UID 映射表、權限重寫 |
| Cgroup | cgroup 掛載點視圖 | 低 — 搭配 cgroupfs |

### xv8-libc 新增

- `setns(fd, nstype) -> isize`
- `unshare(flags) -> isize`
- `sethostname(name, len) -> isize`
- `gethostname(buf, len) -> isize`
- 所有 `CLONE_NEW*` 常數

### xv8-user-std 新增

```rust
// os/linux/namespace.rs
pub struct Namespace(RawFd);

pub enum NamespaceType {
    Cgroup,
    Ipc,
    Network,
    Mount,
    Pid,
    User,
    Uts,
}

impl Namespace {
    pub fn open(pid: u32, nstype: NamespaceType) -> io::Result<Self>;
    pub fn setns(&self) -> io::Result<()>;
    pub fn as_raw_fd(&self) -> RawFd;
}

pub fn unshare(flags: u32) -> io::Result<()>;

// process/Command.rs 擴充
impl Command {
    pub fn unshare_namespace(&mut self, flags: u32) -> &mut Self;
}
```

### QEMU 測試

- `_ns_pid`: clone(CLONE_NEWPID) → getpid 不同
- `_ns_uts`: clone(CLONE_NEWUTS) → sethostname 不影響父
- `_ns_mount`: clone(CLONE_NEWNS) → mount 不影響父
- `_setns`: 子 process 的 /proc/self/ns/pid 傳遞給另一 process 加入

---

## v5.2 — cgroups v2 (Control Groups)

### 目標

實作 Linux cgroups v2，提供 CPU、記憶體、PIDs 等資源限制與監控。

### 架構

```
/sys/fs/cgroup/
├── cgroup.controllers      // 可用控制器
├── cgroup.subtree_control  // 子群組的控制器
├── cpu/
│   ├── cpu.max             // 「quota period」（如 "50000 100000")
│   └── cpu.stat
├── memory/
│   ├── memory.max          // 記憶體上限（bytes）
│   ├── memory.current      // 目前使用量
│   └── memory.stat
└── pids/
    ├── pids.max            // 最大 process 數
    └── pids.current
```

### Kernel 新增

#### cgroupfs (虛擬檔案系統)

| 元件 | 說明 |
|------|------|
| `cgroupfs` | 掛載在 `/sys/fs/cgroup` 的虛擬 FS |
| `cgroup_controller` | CPU、Memory、PIDs 三種控制器 |
| `cgroup_write_proc` | 寫入 `cgroup.procs` 移動 process |
| 階層式 resource tree | 子 cgroup 繼承父限制 |

#### 不需要新 syscall

cgroups v2 完全透過檔案系統介面操作：
- `mount -t cgroup2 none /sys/fs/cgroup`
- `mkdir /sys/fs/cgroup/my_container`
- `echo $PID > /sys/fs/cgroup/my_container/cgroup.procs`
- `echo 50000 100000 > /sys/fs/cgroup/my_container/cpu.max`

### xv8-user-std 新增

```rust
// os/linux/cgroup.rs
pub struct Cgroup {
    path: PathBuf,
}

impl Cgroup {
    pub fn new<P: AsRef<Path>>(name: P) -> io::Result<Self>;
    pub fn add_process(&self, pid: u32) -> io::Result<()>;
    pub fn set_cpu_max(&self, quota: u64, period: u64) -> io::Result<()>;
    pub fn set_memory_max(&self, max: u64) -> io::Result<()>;
    pub fn set_pids_max(&self, max: u32) -> io::Result<()>;
    pub fn memory_current(&self) -> io::Result<u64>;
    pub fn delete(&self) -> io::Result<()>;  // rmdir
}
```

### QEMU 測試

- `_cgroup_basic`: 掛載 cgroupfs、建立 cgroup、寫入 PID、確認限制生效
- `_cgroup_memory`: 設定 `memory.max`、嘗試過量分配、驗證 OOM kill

---

## v5.3 — 安全隔離 (Capabilities + seccomp)

### 目標

提供容器化的安全基礎：Capabilities 實作細粒度權限控制，seccomp 實作系統呼叫過濾。

### Kernel 新增 — Capabilities

| Syscall | 編號 | 功能 |
|---------|------|------|
| `capget` | 142 | `capget(hdrp, datap)` — 讀取 process capability sets |
| `capset` | 143 | `capset(hdrp, datap)` — 設定 process capability sets |

**capability 架構**（比照 Linux）：

```
struct cpu_vfs_cap_data {
    u32 magic_etc;       // VFS_CAP_REVISION_2 | 檔案數
    u32 permitted;       // 允許的 capabilities
    u32 inheritable;     // 可繼承的 capabilities
};
```

每個 process 有 5 組 capability set：
- **Effective (eff)** — 當前生效
- **Permitted (prm)** — 允許提升的
- **Inheritable (inh)** — 可被子 process 繼承
- **Bounding (bnd)** — 子 process 的上限
- **Ambient (amb)** — 非特權程式的 cap 保持

實作至少 20 個常用 capabilities：

| Capability | 說明 |
|------------|------|
| `CAP_CHOWN` | 改變檔案 owner |
| `CAP_DAC_OVERRIDE` | 繞過 DAC 權限檢查 |
| `CAP_NET_RAW` | 使用 raw socket |
| `CAP_NET_BIND_SERVICE` | bind 到特權埠 (< 1024) |
| `CAP_SYS_ADMIN` | 系統管理操作（mount 等） |
| `CAP_SYS_PTRACE` | ptrace 其他 process |
| `CAP_KILL` | 送 signal 到任何 process |
| `CAP_SETUID`/`CAP_SETGID` | 改變 UID/GID |
| `CAP_SYS_CHROOT` | chroot |
| `CAP_NET_ADMIN` | 網路管理（防火牆、介面設定） |

### Kernel 新增 — seccomp

| Syscall | 編號 | 功能 |
|---------|------|------|
| `seccomp` | 144 | `seccomp(op, flags, args)` — 設定 seccomp 策略 |

**seccomp 模式**：

```
SECCOMP_SET_MODE_FILTER (2):
  ┌─────────────────────────────────────┐
  │ prctl(PR_SET_NO_NEW_PRIVS)          │  ← 先呼叫（必須）
  │ seccomp(SECCOMP_SET_MODE_FILTER,    │
  │         SECCOMP_FILTER_FLAG_TSYNC,  │
  │         &prog)                      │
  └─────────────────────────────────────┘
       prog = struct sock_fprog {
           len:    BPF 指令數
           filter: BPF 指令陣列
       }
```

實作簡化版 BPF 過濾器（支援 `cBPF`，非 `eBPF`），足夠實作 Docker 的白名單/黑名單。

**預設容器白名單**（約 50 個安全 syscall）：
```
read, write, close, mmap, munmap, mprotect,
futex, clone, exit, exit_group, gettid,
openat, fstat, statx, readlink,
clock_gettime, nanosleep, sched_yield,
poll, epoll_create1, epoll_ctl, epoll_wait,
pipe2, dup, dup2, dup3, fcntl,
socket, connect, bind, listen, accept,
sendto, recvfrom, sendmsg, recvmsg,
getrandom, capget, capset, prctl, seccomp,
...
```

### xv8-user-std 新增

```rust
// os/linux/capabilities.rs
pub fn cap_get_pid(pid: u32) -> io::Result<Caps>;
pub fn cap_set(caps: &Caps) -> io::Result<()>;

#[repr(C)]
pub struct Caps {
    pub permitted: u64,
    pub inheritable: u64,
    pub effective: u64,
    pub ambient: u64,
}

// os/linux/seccomp.rs
pub fn set_mode_filter(filter: &BpfProgram) -> io::Result<()>;
pub fn set_no_new_privs() -> io::Result<()>;

pub struct BpfProgram {
    pub filter: Vec<sock_filter>,
}
pub struct sock_filter {
    pub code: u16,  // BPF 指令碼
    pub jt: u8,     // jump true
    pub jf: u8,     // jump false
    pub k: u32,     // 通用欄位
}
```

### QEMU 測試

- `_capabilities`: fork 子 process，drop `CAP_NET_RAW`，驗證 socket() 失敗
- `_seccomp`: 載入只允許 `read/write/exit` 的 BPF，驗證 `open()` 被阻擋
- `_seccomp_kill`: 違規 syscall → process 被 SIGKILL

---

## v5.4 — OverlayFS (分層聯合掛載)

### 目標

實作類似 Linux overlayfs 的堆疊式檔案系統，讓多個唯讀層 + 一個讀寫層疊加成單一目錄樹。

這是 Docker image 的基礎：每個 image layer 是唯讀 lowerdir，container 層是讀寫 upperdir。

### 架構

```
container 視角： /mnt/merged
   read-write:  /mnt/upper  (container 層)
   read-only:   /mnt/lower1 (image layer 1)
   read-only:   /mnt/lower2 (image layer 2)
   read-only:   /mnt/lower3 (base image layer)

檔案 lookup 順序： upper → lower1 → lower2 → lower3
寫入（copy-up）：   第一次寫入 lower 檔案時拷貝到 upper
刪除（whiteout）：  在 upper 建立特殊 whiteout 節點
```

### Kernel 新增

| Syscall / 元件 | 功能 |
|----------------|------|
| `mount(overlay)` | `mount("overlay", "/mnt/merged", "overlay", 0, "lowerdir=...,upperdir=...,workdir=...")` |
| `mount()` syscall | xv8 目前無 mount 實作，需實作基本 mount table |

**mount 實作可行性**：xv8 的 FS 是 xv6-style 的 inode-based FS（非 VFS）。要實作 overlayfs，需先引入 VFS 層或直接在 xv6 FS 層 overlay。

**簡化方案**：使用者空間 overlay FUSE（非 kernel 內）
- 優點：不需 VFS 改造
- 缺點：效能較差
- 適合初期原型

### 實作策略

#### 方案 A：Kernel overlayfs（推薦，但較大工程）

1. 先為 xv8 FS 加入 VFS 抽象層：
   ```
   trait Filesystem {
       fn lookup(&self, parent: Inode, name: &str) -> Result<Inode>;
       fn readdir(&self, inode: Inode) -> Result<Vec<DirEntry>>;
       fn read(&self, inode: Inode, offset: u64, buf: &mut [u8]) -> Result<usize>;
       fn write(&self, inode: Inode, offset: u64, buf: &[u8]) -> Result<usize>;
       fn create(&self, parent: Inode, name: &str) -> Result<Inode>;
       fn unlink(&self, parent: Inode, name: &str) -> Result<()>;
       fn mkdir(&self, parent: Inode, name: &str) -> Result<()>;
       fn rmdir(&self, parent: Inode, name: &str) -> Result<()>;
   }
   ```
2. 實作 `OverlayFs` 實例化上述 trait：
   - whiteout 檔案（`c{hk}....` + `trusted.overlay.overlay.whiteout` xattr）
   - copy-up 邏輯
   - 目錄合併（merge dir）

#### 方案 B：User-space overlay（快速原型）

以 xv8-user-std 實作 FUSE-like 的 overlay 掛載工具，攔截檔案操作轉發到實體層。

### xv8-user-std 新增

```rust
// os/linux/overlay.rs (方案 B 快速原型)
pub struct OverlayMount {
    merged: PathBuf,
    lower: Vec<PathBuf>,
    upper: PathBuf,
    work: PathBuf,
}

impl OverlayMount {
    pub fn new(lower: Vec<PathBuf>, upper: PathBuf, work: PathBuf) -> Self;
    pub fn mount(&self, merged: &Path) -> io::Result<()>;
    pub fn unmount(&self) -> io::Result<()>;
}
```

### QEMU 測試

- `_overlay_simple`: 3 層 overlay（2 lower + 1 upper），讀取 lower 檔案、寫入 (copy-up)、確認 upper 有檔案
- `_overlay_delete`: lower 檔案 → merged 刪除 → 確認 whiteout 建立
- `_overlay_dir`: 目錄 merge

---

## v5.5 — 容器網路 (veth pair + bridge + NAT)

### 目標

實作 Linux 風格的 veth pair 虛擬網卡與 bridge，讓 container 擁有獨立網路 namespace 並可對外通訊。

### 架構

```
Host Network Namespace          Container Network Namespace
┌──────────────────────┐      ┌──────────────────────────┐
│   eth0 (E1000)       │      │   eth0@if5 (veth)        │
│   IP: 10.0.2.15     │      │   IP: 172.17.0.2         │
│         │            │      │         │                │
│   xv8-bridge         │──────│   veth1                  │
│   IP: 172.17.0.1    │      │         │                │
│         │            │      │   lo (127.0.0.1)         │
│   NAT (IP forward)  │      │                          │
└──────────────────────┘      └──────────────────────────┘
```

### Kernel 新增

| Syscall / 元件 | 功能 |
|----------------|------|
| `ioctl(SIOCMKETH)` | 自訂 ioctl 建立 veth pair（x v8 無 netlink） |
| Bridge device | 軟體 Layer 2 switch |
| NAT/forward | IP 轉發 + MASQUERADE（簡化版） |

**實作策略（因 xv8 無 netlink）：**

1. 透過 `ioctl` 建立 veth pair：
   ```
   ioctl(fd, SIOCXV8MKVETH, &veth_req)
   // veth_req = { peer_ifname, peer_ns_fd }
   ```
2. 實作軟體 bridge：將多個 interface 加入 bridge
3. NAT 透過簡化版 `iptables` 規則（僅 DNAT/MASQUERADE）

### xv8-user-std 新增

```rust
// os/linux/net.rs
pub struct VethPair {
    pub host_if: String,    // "veth0"
    pub peer_if: String,    // "eth0@if5"
}

impl VethPair {
    pub fn new(host_name: &str, peer_name: &str, peer_ns: Option<&Namespace>) -> io::Result<Self>;
}

pub struct Bridge {
    name: String,
}

impl Bridge {
    pub fn new(name: &str) -> io::Result<Self>;
    pub fn add_interface(&self, if_name: &str) -> io::Result<()>;
    pub fn set_ip(&self, ip: Ipv4Addr, mask: Ipv4Addr) -> io::Result<()>;
    pub fn up(&self) -> io::Result<()>;
}
```

### QEMU 測試

- `_veth`: 建立 veth pair，兩個 namespace 間 ping
- `_bridge`: bridge + 兩個 veth pair，互通
- `_nat_container`: container 透過 NAT 訪問外部（QEMU user-mode NAT 的 10.0.2.0/24）

---

## v5.6 — 容器執行環境 (pivot_root + container lifecycle)

### 目標

整合 v5.1–v5.5，實作完整的容器建立與執行流程。

### Kernel 新增

| Syscall | 編號 | 功能 |
|---------|------|------|
| `pivot_root` | 145 | `pivot_root(new_root, put_old)` — 切換 root FS |
| `sethostname` | 146 | `sethostname(name, len)` — 設定 hostname（UTS ns） |

### 容器啟動流程

```
1. 建立 rootfs（overlayfs mount）
   mount -t overlay -o lowerdir=...,upperdir=...,workdir=... /mnt/container

2. 建立 namespaces
   clone(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWNET | CLONE_NEWUTS)

3. 子 process：
   a. pivot_root("/mnt/container", "/put_old")
   b. umount("/put_old")          // 卸載舊 root
   c. sethostname("container1")
   d. mount -t cgroup2 none /sys/fs/cgroup
   e. setrlimit(RLIMIT_NPROC, ...)
   f. setuid(1000)                // 降權
   g. capset(only KILL, SETUID)  // 限制 capabilities
   h. seccomp(白名單)

4. exec("/bin/sh")
```

### xv8-user-std 工具

```rust
// os/linux/container.rs
pub struct ContainerConfig {
    pub rootfs: OverlayConfig,        // overlay 分層
    pub cmd: Vec<String>,             // 入口命令
    pub hostname: String,             // container hostname
    pub memory_limit: Option<u64>,    // cgroup 記憶體上限
    pub cpu_quota: Option<(u64,u64)>, // cgroup CPU quota/period
    pub pids_limit: Option<u32>,      // cgroup process 數上限
    pub capabilities: Vec<Capability>,// 保留的 capabilities
    pub seccomp_profile: SeccompProfile, // default / permissive
    pub network: NetworkConfig,       // veth / none / host
    pub mounts: Vec<Mount>,           // 額外掛載
}

pub struct Container {
    pub pid: u32,
    pub root_cgroup: Cgroup,
    pub network: Option<VethPair>,
}

impl Container {
    pub fn run(config: ContainerConfig) -> io::Result<Self>;
    pub fn wait(&self) -> io::Result<ExitStatus>;
    pub fn stop(&self) -> io::Result<()>;
    pub fn kill(&self, signal: i32) -> io::Result<()>;
}
```

### QEMU 測試

- `_container_minimal`: 完整 container 生命週期（run → wait → exit code）
- `_container_isolation`: 容器內 PID 1 為 init、hostname 不同
- `_container_resource_limit`: 記憶體限制 → OOM kill

---

## v5.7 — xv8-container 工具

### 目標

提供 Docker-like CLI 工具，方便使用者管理容器。

### 命令列介面

```
xv8-container pull   busybox:latest     # 下載 rootfs
xv8-container run    -it busybox sh      # 執行容器
xv8-container exec   <container> sh     # 進入運行中容器
xv8-container ps                        # 列出容器
xv8-container rm     <container>        # 刪除容器
xv8-container images                    # 列出 image
xv8-container commit <container> <name> # 建立新 image
```

### 實作要點

- 容器狀態儲存於 `/var/lib/xv8-container/containers/`（JSON）
- Image rootfs 儲存於 `/var/lib/xv8-container/images/`
- 每個 container 目錄下有 `config.json`, `rootfs/`（overlay mount point）
- 支援 overlay layers 的匯入/匯出（tar 格式）

### QEMU 測試

- 完整 E2E 測試：`xv8-container run alpine echo hello` → 輸出 hello

---

## v5.x 實作依賴關係

```
v5.1 Namespaces ──────────────┐
     │                        │
v5.2 cgroups  ───────┐       │
     │                │       │
v5.3 Cap/seccomp ──┐ │       │
     │               │ │       │
v5.4 OverlayFS ───┐ │ │       │
     │              │ │ │       │
v5.5 Veth/Bridge ─┐│ │ │       │
     │             ││ │ │       │
v5.6 Container ───│││ │ │       │
     │             │││ │ │       │
v5.7 xv8-container │││ │ │       │
     │             │││ │ │       │
     v4.1 eventfd ─┘││ │ │       │
     v4.2 memfd ────┘│ │ │       │
     v4.3 splice ────┘ │ │       │
     v4.4 prctl ────────┘ │       │
     v4.5 inotify ────────┘       │
     v4.6 os::linux ──────────────┘
```

v5.1 可獨立進行（僅依賴 v4.2 pidfd）。
v5.2–v5.5 互不依賴，可平行開發。
v5.6 整合 v5.1–v5.5。
v5.7 依賴 v5.6。

---

## 相關檔案結構

```
xv8/kernel/src/
├── namespace.rs     // v5.1: namespace 管理
├── cgroup.rs        // v5.2: cgroup controller
├── capability.rs    // v5.3: capability sets
├── seccomp.rs       // v5.3: BPF filter
├── overlay.rs       // v5.4: overlay filesystem
├── veth.rs          // v5.5: virtual ethernet
├── bridge.rs        // v5.5: software bridge
└── syscall.rs       // 新增 syscall 分發

xv8rust/xv8-libc/src/
└── raw.rs           // 新 syscall wrappers

xv8rust/xv8-user-std/src/os/linux/
├── namespace.rs     // v5.1
├── cgroup.rs        // v5.2
├── capabilities.rs  // v5.3
├── seccomp.rs       // v5.3
├── overlay.rs       // v5.4
├── net.rs           // v5.5
└── container.rs     // v5.6

xv8/user/tools/
└── xv8-container.rs // v5.7
```

---

## Syscall 編號分配 (v5.x)

| 編號 | Syscall | 版本 |
|------|---------|------|
| 140 | `setns` | v5.1 |
| 141 | `unshare` | v5.1 |
| 142 | `capget` | v5.3 |
| 143 | `capset` | v5.3 |
| 144 | `seccomp` | v5.3 |
| 145 | `pivot_root` | v5.6 |
| 146 | `sethostname` | v5.6 |

v5.2 (cgroups) 與 v5.4 (OverlayFS) 透過 `mount` + 檔案系統操作實現，不需新 syscall。
v5.5 (veth) 透過 `ioctl` + 虛擬網路設備實現。

---

> 最後更新：2026-06-07
> 起始版本：v4.1 | v4.x 目標完成：v4.7 | v5.x 目標完成：v5.7
