# TFTP — 簡易檔案傳輸協定

`tftp.rs` 實作 TFTP（Trivial File Transfer Protocol）客戶端與伺服器功能。

## TFTP 協定基礎

TFTP（RFC 1350）是一種極簡的檔案傳輸協定，使用 UDP 埠 69。與 FTP 不同，TFTP 無認證、無目錄瀏覽、僅支援基本檔案讀寫。其簡單性使其適合用於嵌入式系統與網路裝置的韌體更新。

## 封包類型

| Opcode | 類型 | 說明 |
|--------|------|------|
| 1 | RRQ | Read Request（讀取請求） |
| 2 | WRQ | Write Request（寫入請求） |
| 3 | DATA | 資料封包（含區塊編號） |
| 4 | ACK | 確認封包 |
| 5 | ERROR | 錯誤訊息 |

## 鎖步協定

TFTP 使用鎖步（lock-step）傳輸：發送一個 DATA 封包後等待 ACK 再發送下一塊。區塊編號從 1 開始，小於 512 位元組的封包表示傳輸結束。

## 相關文件

- [tftp.md](../../tools/src/bin/tftp.md) — TFTP 工具
- [udp.md](../../kernel/src/net/udp.md) — UDP 傳輸
