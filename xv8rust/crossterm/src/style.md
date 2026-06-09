# 樣式設定模組

管理終端文字的前景色、背景色與文字屬性（粗體、斜體、底線、閃爍、反轉等）。
透過 `SetAttr`、`SetFg`、`SetBg` 等 Command 實作，使用 ANSI SGR
（Select Graphic Rendition）序列。支援 3/4 位色、8 位 256 色與 24 位真彩色。
提供 `ContentStyle` 結構體作為豐富樣式容器，支援鏈式呼叫與疊加效果。
此模組決定了輸出文字在終端中的視覺呈現。
