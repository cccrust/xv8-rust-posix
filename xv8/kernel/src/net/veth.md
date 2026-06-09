# Veth Pair 模組 — veth.rs

## 理論背景

Veth (Virtual Ethernet) pair 是一對虛擬網路卡，一端傳送的封包會自動出現在另一端，類似 Linux 的 `veth` 裝置。Veth pair 是容器網路的核心元件：

- 一端放在主機的 net namespace，一端放在容器的 net namespace
- 封包透過 veth pair 在 namespace 之間傳遞
- 可搭配 bridge 實現多容器互通

## xv8 實作

### 資料結構

```rust
pub struct VethEndpoint {
    pub pair_id: usize,        // pair 識別碼
    pub peer_dev: Arc<Mutex<dyn NetDevice>>,  // 另一端
    pub recv_queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
    pub dev_name: [u8; 16],    // 裝置名稱
}
```

`VethEndpoint` 實現 `NetDevice` trait：
```rust
impl NetDevice for VethEndpoint {
    fn send(&self, data: &[u8]) -> Result<(), NetError>;
    fn recv(&self, buf: &mut [u8]) -> Result<usize, NetError>;
    fn mac_addr(&self) -> MacAddr;
    fn dev_type(&self) -> DeviceType;
}
```

### ioctl 介面

```rust
pub fn ioctl_create_veth(arg: VA) -> Result<usize, SysError>;
```

使用 `ioctl` 命令 `XV8_VETH_CREATE = 100` 在 socket fd 上建立 veth pair。

### 建立流程

1. 使用者呼叫 `ioctl(sock_fd, XV8_VETH_CREATE, ...)`
2. 核心配置兩個 `VethEndpoint`，互為 peer
3. 一端註冊到主機的 loopback/網路堆疊
4. 回傳成功

## 系統呼叫

xv8 使用 `ioctl` 而非專用系統呼叫建立 veth pair：

| 命令 | 值 | 說明 |
|------|-----|------|
| `XV8_VETH_CREATE` | 100 | 在 socket fd 上建立 veth pair |

## 相關文件

- [Wiki: 容器](../../../../_wiki/Container.md)
- [Wiki: Namespace](../../../../_wiki/Namespace.md)
- [network 總覽](mod.md)
