# Cksum — 檔案 CRC 檢查碼

`cksum` 計算檔案的 CRC（Cyclic Redundancy Check）檢查碼與位元組數。實作使用 CRC-32 演算法（POSIX 1003.2-1992 定義的版本），產生 32-bit 檢查碼。CRC 是資料完整性的基本工具，可偵測傳輸或儲存過程中的隨機錯誤。不同於加密雜湊（SHA-256），CRC 易於偽造，不適用於安全場景。

## 相關文件

- [sum.md](./sum.md) — 傳統 BSD/System V checksum
