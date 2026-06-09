# Mod — 協定資料結構模組

`mod.rs` 是 `proto` 子模組的根，組織協定層的資料結構定義。

## 設計原則

`proto` 子模組遵循關注點分離原則：

- **proto/**: 純資料結構，定義協定封包的 bytes ↔ struct 轉換
- **上層 (dns.rs, icmp.rs 等)**: 協定的應用邏輯，使用 `proto` 的資料結構

## 子模組

- **dns**: DNS 封包結構（標頭、問題、資源記錄）
- **icmp**: ICMP 封包結構（Echo、Error 訊息）
- **util**: 協定層共用函式（位元組緩衝區操作、位元組序）

## 相關文件

- [mod.md（上層）](../lib.md) — libnet 總覽
- [dns.md](./dns.md) — DNS 資料結構
- [icmp.md](./icmp.md) — ICMP 資料結構
- [util.md](./util.md) — 協定層工具
