# IPv4 — 網際網路協定第 4 版

IPv4 是網路層協定，負責跨網路的封包轉送。

## 封包格式

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┬─┬─┬─┬─┬─┬─┬─┼─┼─┬─┬─┬─┬─┬─┼─┼─┼─┬─┼─┼─┼─┬─┼─┼─┼─┬─┼─┼─┼─┬─┤
│Version│  IHL  │    TOS   │         Total Length            │
├───────┼───────┼──────────┼───────────┬─────────────────────┤
│      Identification     │Flags│      Fragment Offset    │
├───────┼───────┼──────────┼───────────┴─────────────────────┤
│  TTL  │ Protocol│          Header Checksum             │
├───────┼───────┼──────────┼───────────────────────────────┤
│                      Source IP Address                  │
├─────────────────────────────────────────────────────────┤
│                   Destination IP Address               │
├─────────────────────────────────────────────────────────┤
│                    Options (if IHL > 5)                 │
└─────────────────────────────────────────────────────────┘
```

## xv8 IPv4 標頭

```rust
#[repr(C, packed)]
pub struct Ipv4Header {
    ver_ihl: u8,        // 版本(4) + IHL(5) = 0x45
    tos: u8,              // Type of Service
    len: Be<u16>,         // 總長度
    id: Be<u16>,          // 識別符
    off: Be<u16>,         // Flags + Fragment Offset
    ttl: u8,              // Time to Live (64)
    proto: u8,            // 協定 (1=ICMP, 6=TCP, 17=UDP)
    sum: Be<u16>,         // 標頭校驗和
    src: Ipv4Addr,        // 來源 IP
    dest: Ipv4Addr,       // 目的地 IP
}
```

## Protocol 欄位

| 值 | 協定 |
|----|------|
| 1 | ICMP |
| 6 | TCP |
| 17 | UDP |

## TTL

防止封包在網路中無限循環，每經過一個路由器減 1。xv8 預設 TTL = 64。

## 校驗和計算

```rust
fn calculate_checksum(&self) -> u16 {
    let mut header = *self;
    header.sum = Be::new(0);
    net::internet_checksum(&[header.as_bytes()])
}
```

## 分片處理

xv8 預設不支援分片。`off` 欄位設為 0。

## 路由決策

```rust
pub fn handle_ipv4(interface_id: InterfaceId, packet: &[u8]) -> Result<(), NetError> {
    let Some((req_ipv4, req_data)) = Ipv4Header::from_bytes_with_rest(packet) else {
        err!(NetError::MalformedPacket)
    };

    // 版本和 IHL 檢查
    if ver != 4 || ihl != 5 {
        err!(NetError::MalformedPacket);
    }

    // 校驗和檢查
    if internet_checksum(&[header_without_sum.as_bytes()]) != req_ipv4.sum.get() {
        err!(NetError::ChecksumFailed);
    }

    // 分派到上層協定
    match req_ipv4.proto() {
        Ipv4Proto::Icmp => log!(icmp::handle_icmp(req_ipv4.src, req_data)),
        Ipv4Proto::Tcp => log!(handle_tcp(req_ipv4.src, req_ipv4.dest, req_data)),
        Ipv4Proto::Udp => log!(udp::handle_udp(interface_id, req_ipv4.dest, req_ipv4.src, req_data)),
        Ipv4Proto::Unknown => Ok(()),
    }
}
```

## 地址類型

```rust
impl Ipv4Addr {
    pub const UNSPECIFIED: Self = Self([0; 4]);
    pub const BROADCAST: Self = Self([255; 4]);
    pub const LOOPBACK: Self = Self([127, 0, 0, 1]);
}
```

## 私有位址範圍

| 範圍 | 用途 |
|------|------|
| 10.0.0.0/8 | 私有網路 |
| 172.16.0.0/12 | 私有網路 |
| 192.168.0.0/16 | 私有網路 |

## 子網路遮罩

```rust
pub fn prefix_len_to_mask(prefix_len: u8) -> Option<Self> {
    if prefix_len == 0 {
        Some(Self([0; 4]))
    } else if prefix_len <= 32 {
        let mask = u32::MAX << (32 - prefix_len as u32);
        Some(Self(mask.to_be_bytes()))
    } else {
        None
    }
}

pub fn mask_to_prefix_len(mask: Self) -> Option<u8> {
    let mask = u32::from_be_bytes(mask.0);
    let prefix_len = mask.leading_ones() as u8;
    // 驗證是否為連續 1s
}
```

## 與上層的關係

```
IPv4 Header
    │
    ├── proto = 1 → ICMP
    ├── proto = 6 → TCP
    └── proto = 17 → UDP
```

## QEMU 環境

QEMU user-mode NAT 提供：
- IP: 10.0.2.15（分配給 xv8）
- Gateway: 10.0.2.2
- DNS: 10.0.2.3

## 相關主題

- [[Ethernet]]：資料連結層
- [[ARP]]：IP-MAC 解析
- [[Routing]]：路由表