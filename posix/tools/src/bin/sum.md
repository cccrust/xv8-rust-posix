# Sum — 檔案檢查碼

`sum` 計算檔案的 checksum（檢查碼）與區塊數。支援 BSD 校驗和（`-r`，16-bit 循環和）與 System V 校驗和（`-s`，32-bit 和）。不同於 `cksum`（使用 CRC-32），`sum` 使用更簡單的加法演算法，適用於傳統檔案完整性檢查。

## 相關文件

- [cksum.md](./cksum.md) — CRC-32 檢查碼
- [wc.md](./wc.md) — 字數統計
