# Tabs — 設定終端機定位

`tabs` 設定終端機的硬體定位（tab stop）位置。傳統終端機（如 VT100）支援可程式設定的定位點。`tabs -a` 設定 ANSI 標準定位（每 8 欄），`tabs -d` 設定等距定位。現代終端機軟體定位由驅動處理，但此命令保留歷史相容性。

## 相關文件

- [stty.md](./stty.md) — 終端機設定
- [tput.md](./tput.md) — 終端機能力
- [expand.md](./expand.md) — Tab 轉換為空白
