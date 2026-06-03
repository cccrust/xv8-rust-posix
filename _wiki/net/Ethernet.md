# Ethernet — 乙太網路層

Ethernet 是資料連結層協定，負責在同一網路區域內傳遞框架。

## 框架結構

```
┌──────────┬──────────┬──────────┬─────────┬────────────┐
│   DST    │   SRC    │ EtherType│  Data   │    CRC     │
│  (6B)    │  (6B)   │  (2B)    │         │   (4B)     │
└──────────┴──────────┴──────────┴─────────┴────────────┘
         ◄─────────── 14 bytes ──────────►
```

## EtherType 編碼

| EtherType | 協定 |
|-----------|------|
| 0x0800 | IPv4 |
| 0x0806 | ARP |
| 0x86DD | IPv6 |

## xv8 實作

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EthernetHeader {
    pub dst: MacAddr,
    pub src: MacAddr,
    pub ether_type: Be<u16>,
}

pub enum EtherType {
    Ipv4 = 0x0800,
    Arp = 0x0806,
    Unknown = u16::MAX,
}
```

## MAC 位址

```rust
#[repr(transparent)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    const UNSPECIFIED: Self = Self([0x00; 6]);
    const BROADCAST: Self = Self([0xff; 0xff, 0xff, 0xff, 0xff, 0xff]);
}
```

廣播 MAC 位址：`ff:ff:ff:ff:ff:ff`

## 框架處理

```rust
fn receive(id: InterfaceId, packet: Box<[u8]>) -> Result<(), NetError> {
    let Some((eth, data)) = EthernetHeader::from_bytes_with_rest(&packet) else {
        err!(NetError::MalformedPacket);
    };

    match eth.ether_type() {
        EtherType::Arp => log!(arp::handle_arp(id, eth, data)),
        EtherType::Ipv4 => log!(ipv4::handle_ipv4(id, data)),
        EtherType::Unknown => Ok(()),
    }
}
```

## 回覆框架

```rust
pub fn new_reply(request: &EthernetHeader, src: MacAddr) -> Self {
    Self {
        dst: request.src,        // 交換來源和目的
        src,
        ether_type: request.ether_type,
    }
}
```

## 傳送流程

```rust
let eth = EthernetHeader::new(MacAddr::UNSPECIFIED, src_mac, EtherType::Ipv4);
```

目的地 MAC 未指定時（需要 ARP 解析），填入 `UNSPECIFIED`。

## MTU

乙太網路標準 MTU 為 1500 位元組，不包括 Ethernet header。

## CRC 校驗

硬體（網卡）負責計算和驗證 CRC。軟體通常假設 CRC 已由硬體處理。

## 廣播

乙太網路層級廣播使用 MAC 廣播位址，所有裝置都會接收。

## 與上層的關係

```
Ethernet Header
    │
    ├── EtherType = 0x0800 → IPv4
    │
    ├── EtherType = 0x0806 → ARP
    │
    └── EtherType = 其他 → 忽略
```

## 安全性考量

- MAC 位址可以被偽造（MAC spoofing）
- 交換機防止部分問題
- 需要上層協定（IP、ARP）驗證