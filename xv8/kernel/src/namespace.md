# Namespace 模組 — namespace.rs

## 理論背景

Namespace (命名空間) 是 Linux 核心提供的資源隔離機制，將全域系統資源包裝在抽象層中，讓一組 process 只能看到所屬 namespace 內的資源。xv8 實作完整的 7 類型 namespace 支援，作為容器隔離的基礎。

Namespace 的設計源自作業系統的「最小權限原則」(principle of least privilege) ── process 應該只看見它需要的資源。Plan 9 作業系統在 1990 年代率先實作類似概念，Linux 2002 年在 2.4.19 引入 mount namespace，逐步擴展到 7 種類型。

## xv8 實作

### NsType 枚舉

Namespace 類型以 `NsType` 枚舉表示，與對應的 `CLONE_NEW*` 旗標透過 `nstype_to_flag()` 映射：

```rust
pub enum NsType {
    Mount = 0,  Cgroup = 1,  Uts = 2,  Ipc = 3,
    User = 4,   Pid = 5,     Net = 6,
}
```

### NsProxy

所有 namespace 類型被包裝在 `NsProxy` 結構中，每個類型使用 `Arc` 實現共用：

```rust
pub struct NsProxy {
    pub pid: Arc<PidNamespace>,
    pub uts: Arc<UtsNamespace>,
    pub mount: Arc<MountNamespace>,
    pub net: Arc<NetNamespace>,
    pub ipc: Arc<IpcNamespace>,
    pub user: Arc<UserNamespace>,
    pub cgroup: Arc<CgroupNamespace>,
}
```

### 建立與繼承

`from_parent()` 根據旗標決定哪些 namespace 應建立新實例、哪些繼承自父 process：

```rust
pub fn from_parent(parent: &NsProxy, flags: usize) -> Self {
    // 若 CLONE_NEWPID 設置，建立新的 PidNamespace
    // 否則共用父 process 的 PidNamespace
}
```

### nsopen + setns

xv8 定義 `nsopen(pid, nstype)` 系統呼叫 (150) 以取得 namespace fd。`NsFd` 包含目標 process 的完整 `NsProxy`，`setns` 使用 `clone_with_override()` 僅取代指定的 namespace 類型。

### NamespaceId

每個 namespace 實例具有唯一的 `NamespaceId`，使用全域原子計數器分配：

```rust
pub struct NamespaceId(usize);
impl NamespaceId {
    pub fn alloc() -> Self { ... }
}
```

## 相關文件

- [Wiki: Namespace](../../../_wiki/Namespace.md)
- [Wiki: 容器](../../../_wiki/Container.md)
- [syscall 文件](syscall.md)
- [proc 文件](proc.md)
