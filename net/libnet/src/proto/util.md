# Util — 協定層工具函式

`util.rs` 提供協定層共用的資料操作工具，包括緩衝區讀寫、位元組序轉換與檢查碼計算。

## 功能

### 緩衝區操作

- `read_u16(buf)`: 從緩衝區讀取大端序 16-bit 無號整數
- `write_u16(buf, val)`: 將 16-bit 值以大端序寫入緩衝區
- `read_u32(buf)`: 從緩縮區讀取大端序 32-bit 無號整數
- `write_u32(buf, val)`: 將 32-bit 值以大端序寫入緩衝區

### 網域名稱編碼解碼

- `encode_domain_name(name)`: 將 DNS 域名（如 `www.example.com`）編碼為 label 序列
- `decode_domain_name(buf, offset)`: 從緩衝區解碼 DNS 域名，支援 pointer 解析

### 檢查碼

- `internet_checksum(data)`: 計算 RFC 1071 Internet checksum

## 相關文件

- [mod.md](./mod.md) — 協定子模組
- [dns.md](./dns.md) — DNS 資料結構
- [util.md](../util.md) — libnet 共用工具
