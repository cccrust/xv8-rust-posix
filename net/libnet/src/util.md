# Util — 共用工具函式

`util.rs` 提供 libnet 中各協定實作所需的共用工具函式，包括位址解析、位元組序轉換、位元操作等。

## 實作的功能

### 位址轉換

- `ip_to_u32` / `u32_to_ip`: 點分十進位字串與 32-bit 整數的轉換
- `mac_str_to_bytes`: MAC 位址字串（`xx:xx:xx:xx:xx:xx`）轉位元組陣列
- `resolve_host`: DNS 名稱解析

### 位元組序轉換

網路協定使用大端序（big-endian / network byte order）。工具函式確保主機位元組序與網路位元組序之間的正確轉換：

- `htons` / `ntohs`: 16-bit 主機至網路 / 網路至主機
- `htonl` / `ntohl`: 32-bit 版本

### 檢查碼計算

- `checksum`: 16-bit Internet checksum（RFC 1071）

## 相關文件

- [lib.md](./lib.md) — libnet 總覽
- [net_impl.md](./net_impl.md) — 網路操作實作
- [util.md](../proto/util.md) — 協定層工具函式
