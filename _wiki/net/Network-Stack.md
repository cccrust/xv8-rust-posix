# Network-Stack（網路堆疊）

xv8 實現了一個完整的網路堆疊，包含乙太網路、ARP、IPv4、UDP、DHCP 和 ICMP，以及 Intel E1000 網卡的驅動程式。

## 網路堆疊分層

```
┌────────────────────────────────────────┐
│         應用層                          │
│    (dns, ping, netcat tools)           │
├────────────────────────────────────────┤
│         UDP / ICMP                    │
├────────────────────────────────────────┤
│         IPv4                          │
├────────────────────────────────────────┤
│         ARP                           │
├────────────────────────────────────────┤
│         Ethernet                      │
├────────────────────────────────────────┤
│    E1000 Driver / VirtIO              │
└────────────────────────────────────────┘
```

## 初始化序列

網路堆疊在核心啟動時按照依賴順序初始化：

1. `net/mod.rs` 的 `net::init()` 是入口點
2. 初始化 `ethernet`（乙太網路層）
3. 初始化 `arp`（位址解析協定）
4. 初始化 `ipv4`（網際網路協定）
5. 初始化 `dhcp`（動態主機設定）
6. 初始化 `udp` 和 `icmp`（傳輸/應用層協定）
7. 初始化介面管理

## Ethernet（乙太網路層）

乙太網路是資料連結層協定。xv8 的乙太網路程式碼在 `net/eth.rs`：

```rust
pub struct EthHdr {
    pub dhost: [u8; 6],       // 目的地 MAC 位址
    pub shost: [u8; 6],       // 來源 MAC 位址
    pub ether_type: u16,       // 上層協定類型
}
```

常見的 EtherType：
- 0x0800：IPv4
- 0x0806：ARP
- 0x86DD：IPv6

當接收乙太網路框時，核心根據 `ether_type` 分派到ARP、IPv4 或其他處理常式。

## ARP（位址解析協定）

ARP 將 IP 位址解析為 MAC 位址。在乙太網路環境中通訊需要知道目標的 MAC 位址。

```rust
pub struct ArpHdr {
    pub hw_type: u16,         // 硬體類型（1 為乙太網路）
    pub proto_type: u16,       // 協定類型（0x0800 為 IPv4）
    pub hw_size: u8,          // 硬體位址長度（6 for Ethernet）
    pub proto_size: u8,       // 協定位址長度（4 for IPv4）
    pub opcode: u16,           // 操作（1=請求, 2=回應）
}
```

ARP 表（`arp.rs` 中的 `ArpCache`）快取 IP-MAC 對應。如果目標 IP 不在快取中，則發送 ARP 請求並等待回應。

## IPv4（網際網路協定第 4 版）

xv8 實現了一個基本的 IPv4 堆疊（`net/ipv4.rs`）：

```rust
pub struct Ipv4Hdr {
    pub version_ihl: u8,      // 版本（4）和 header長度
    pub tos: u8,               // 服務類型
    pub total_len: u16,        // 總長度
    pub ident: u16,            // 識別符
    pub flags_frag: u16,       // 旗標和片段偏移
    pub ttl: u8,               // 存活時間
    pub proto: u8,             // 上層協定（1=ICMP, 6=TCP, 17=UDP）
    pub checksum: u16,         // 標頭校验和
    pub src: u32,              // 來源 IP
    pub dst: u32,              // 目的地 IP
}
```

IPv4 負責：
- 分片與重組
- 路由轉發
- 校驗和驗證

## UDP（用戶資料報協定）

UDP 是不連接導向的傳輸層協定（`net/udp.rs`）：

```rust
pub struct UdpHdr {
    pub src_port: u16,        // 來源連接埠
    pub dst_port: u16,         // 目的地連接埠
    pub length: u16,           // UDP 資料報長度
    pub checksum: u16,         // 校驗和（可選）
}
```

xv8 的 UDP 實現：
- 支援發送和接收 UDP 資料報
- 進行校驗和驗證
- 通過連接埠號路由到應用程式

## DHCP（動態主機設定協定）

DHCP（`net/dhcp.rs`）允許 xv8 自動取得網路設定：

1. 發送 DHCPDISCOVER 廣播
2. 接收 DHCPOFFER（伺服器提供 IP）
3. 發送 DHCPREQUEST（請求該 IP）
4. 接收 DHCPACK（確認）

DHCP 交互使用 UDP 連接埠 67（伺服器）和 68（客戶端）。

在 QEMU 環境中，QEMU 的內建 DHCP 伺服器（10.0.2.3）會回應：

```rust
pub const QEMU_DHCP_SERVER: u32 = 0x020000a;  // 10.0.2.3
pub const QEMU_GATEWAY: u32 = 0x020000a;       // 10.0.2.2
```

## ICMP（網際網路控制訊息協定）

ICMP（`net/icmp.rs`）用於錯誤報告和網路偵測：

```rust
pub struct IcmpHdr {
    pub icmp_type: u8,        // 類型
    pub code: u8,              // 程式碼
    pub checksum: u16,        // 校驗和
}
```

xv8 支援：
- Echo Request（類型 8）：ping 使用
- Echo Reply（類型 0）：ping 回應
- Destination Unreachable（類型 3）

## E1000 網卡驅動

Intel E1000 是 QEMU 模擬的網卡（`e1000.rs`）。它是一個 PCIe 裝置，需要：

1. **PCI 枚舉**：發現 E1000（Vendor ID 0x8086，Device ID 0x100E）
2. **記憶體映射 I/O**：E1000 使用 MMIO（記憶體映射 I/O）
3. **DMA**：直接記憶體存取，用於接收和發送框
4. **中斷處理**：接收完成或發送完成時產生中斷

E1000 使用描述符環（descriptor rings）來管理 Rx/Tx DMA 緩衝區。

## 路由表

`net/route.rs` 實現了簡單的路由表：

```rust
pub struct RouteEntry {
    pub dest: u32,             // 目的地網路
    pub mask: u32,             // 網路遮罩
    pub gateway: u32,          // 閘道器 IP
    pub iface: u32,            // 輸出介面
}
```

路由查詢時，目標 IP 與每個條目的遮罩進行 AND 運算來確定目的地網路。

## 介面管理

`net/interface.rs` 管理網路介面：

```rust
pub struct NetInterface {
    pub name: &'static str,    // 介面名稱（如 "eth0"）
    pub mac: [u8; 6],          // MAC 位址
    pub ip: u32,               // IP 位址
    pub netmask: u32,          // 網路遮罩
    pub gateway: u32,          // 預設閘道
    pub mtu: u16,              // 最大傳輸單元（1500）
}
```

xv8 支援：
- 主要網路介面（eth0，由 E1000 提供）
- 迴環介面（127.0.0.1，用於本機通訊）

## 迴環介面

`net/loopback.rs` 實現了迴環介面：

- 目的地為本機網路的 IP 的封包被路由到迴環介面
- 封包直接被接收並重定向到本機的 UDP/TCP 處理

## 網路工具

xv8 使用者空間的網路工具（`user/bin/`）包括：

- `dns.rs`：DNS 查詢（向 10.0.2.3 發送 UDP DNS 查詢）
- `ping.rs`：ICMP echo request/reply
- `udp.rs`：UDP 傳送/接收測試
- `traceroute.rs`：路徑追蹤

## QEMU 網路設定

在 QEMU 中，網路通過使用者模式 NAT 設定：

```
-netdev user,id=net0 -device e1000,netdev=net0
```

這創建了一個虛擬網路：
- xv8 獲得 IP（如 10.0.2.15）
- 10.0.2.2 是閘道器
- 10.0.2.3 是 DNS 伺服器

## 相關主題

- [[Device-Drivers]]：E1000 驅動程式
- [[Syscall]]：socket 系統呼叫
- [[Process]]：網路程式的程序管理