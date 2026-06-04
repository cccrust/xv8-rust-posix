# ARP — 位址解析協定

ARP 將 IP 位址解析為 MAC 位址，是 IPv4 網路的關鍵協定。

## 運作原理

```
主機 A (192.168.1.10)              主機 B (192.168.1.20)
       │                                    │
       │  需要傳送到 192.168.1.20           │
       │  但不知道 MAC 位址                  │
       ▼                                    │
  ┌─────────────────────────────────────┐    │
  │  查詢 ARP 快取                       │    │
  │  找到 → 直接傳送                    │    │
  │  未找到 → 發送 ARP 請求              │    │
  └─────────────────────────────────────┘    │
       │                                    │
       │  ARP Request (廣播)                  │
       │  誰有 192.168.1.20?                │
       ├──────────────────────────────────►│
       │                                    │
       │  (所有主機都收到)                   │
       │                                    ▼
       │                              確認是否為自己的 IP
       │                                    │
       │  ARP Reply (單播)                   │
       │  我的 MAC 是 aa:bb:cc:dd:ee:ff    │
       │◄──────────────────────────────────┤
       │                                    │
       ▼                                    ▼
  更新 ARP 快取                             更新 ARP 快取
```

## xv8 ARP 實作

### ARP 標頭

```rust
#[repr(C, packed)]
struct ArpPacket {
    htype: Be<u16>,      // 硬體類型 (1 = Ethernet)
    ptype: Be<u16>,      // 協定類型 (0x0800 = IPv4)
    hlen: u8,            // 硬體位址長度 (6)
    plen: u8,            // 協定位址長度 (4)
    op: Be<u16>,         // 操作 (1=請求, 2=回應)
    sha: MacAddr,        // 發送者硬體位址
    spa: Ipv4Addr,       // 發送者協定位址
    tha: MacAddr,        // 目標硬體位址
    tpa: Ipv4Addr,       // 目標協定位址
}
```

### ARP 快取

```rust
const ARP_CACHE_SIZE: usize = 64;

static ARP_CACHE: SpinLock<ArpCache> = SpinLock::new(
    ArpCache {
        entries: [None; ARP_CACHE_SIZE],
        eviction_index: 0,
    },
    "arp_cache",
);

impl ArpCache {
    pub fn lookup(ip: Ipv4Addr) -> Option<MacAddr>;
    fn insert(ip: Ipv4Addr, mac: MacAddr);
}
```

固定大小陣列，線性搜尋。

### ARP 請求

```rust
pub fn request(interface_id: InterfaceId, dest_ip: Ipv4Addr) -> Result<(), NetError> {
    // Ethernet header: 目的地 = 廣播 MAC
    // ARP Request 封包
}
```

### 處理 ARP 回應

```rust
pub fn handle_arp(...) -> Result<(), NetError> {
    match req_arp.op() {
        ArpOp::Request => {
            try_log!(reply(interface_id, req_eth, req_arp));
        }
        ArpOp::Response => {
            ArpCache::insert(req_arp.spa, req_arp.sha);
            OutgoingQueue::dispatch(req_arp.spa, req_arp.sha);
        }
    }
    Ok(())
}
```

收到回應後，更新快取並發送排隊的封包。

## 代理 ARP

xv8 目前未實現代理 ARP。

## 安全性考量

### ARP 欺騙

攻擊者偽造 ARP Reply：
```
正常：主機 B 回應 "192.168.1.20 是 aa:bb:cc:dd:ee:ff"
攻擊：攻擊者回應 "192.168.1.20 是 11:22:33:44:55:66"
```

### 防護措施

- 靜態 ARP 項目
- ARP 監控
- 交換機的 Dynamic ARP Inspection

## 與 IPv6 的關係

IPv6 使用 NDP（Neighbor Discovery Protocol）代替 ARP。

## 封包格式

```
乙太網路 Header:
  DST: ff:ff:ff:ff:ff:ff (廣播)
  SRC: aa:bb:cc:dd:ee:ff
  Type: 0x0806 (ARP)

ARP Request:
  HTYPE: 1 (Ethernet)
  PTYPE: 0x0800 (IPv4)
  HLEN: 6
  PLEN: 4
  OP: 1 (Request)
  SHA: aa:bb:cc:dd:ee:ff
  SPA: 192.168.1.10
  THA: 00:00:00:00:00:00
  TPA: 192.168.1.20
```

## 計時

ARP 快取項目應有計時，通常 20 分鐘後過期。xv8 目前無計時機制。

## 相關主題

- [[Ethernet]]：乙太網路層
- [[IPv4]]：IP 層