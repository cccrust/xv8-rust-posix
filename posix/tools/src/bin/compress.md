# Compress — 資料壓縮

`compress` 使用 LZW（Lempel-Ziv-Welch）演算法壓縮檔案，產生 `.Z` 結尾的壓縮檔案。LZW 是目前 GIF 與早期 Unix compress 使用的字典式壓縮演算法。它利用輸入資料的重複模式，將重複字串序列替換為較短的字典索引。

## 相關文件

- [uncompress.md](./uncompress.md) — 解壓縮
- [zcat.md](./zcat.md) — 檢視壓縮檔案
