# TCP — 傳輸控制協定（RFC 9293）

## 概述

TCP 提供**可靠、有序、面向連線**的位元組串流服務，是 HTTP、SSH 等協定的基礎。

## 連線狀態機

```
CLOSED → LISTEN → SYN_RCVD → ESTABLISHED → ...
CLOSED → SYN_SENT → ESTABLISHED → FIN_WAIT1 → FIN_WAIT2 → TIME_WAIT → CLOSED
ESTABLISHED → CLOSE_WAIT → LAST_ACK → CLOSED
```

## TCP 區段格式

| 欄位 | 長度 | 說明 |
|------|------|------|
| 來源/目標埠 | 16+16 bits | 端點識別 |
| Sequence Number | 32 bits | 資料流序號 |
| Acknowledgment Number | 32 bits | 確認序號 |
| Flags | 9 bits | SYN/ACK/FIN/RST 等 |
| Window Size | 16 bits | 流量控制窗口 |

## 三向交握

```
SYN (seq=x)       →
                   ← SYN+ACK (seq=y, ack=x+1)
ACK (seq=x+1)     →
```

## 流量/壅塞控制

- **流量控制**: Window Size 避免接收端緩衝區溢出
- **壅塞控制**: 慢啟動、壅塞避免、快速重傳

## 相關文件

- [ipv4.md](./ipv4.md) — IP 封裝
- [udp.md](./udp.md) — UDP 對比
- [sysnet.md](../sysnet.md) — 系統呼叫
