# 文字屬性模組

定義終端文字屬性的列舉與操作，對應 ANSI SGR 參數。支援的屬性包括：
`Bold`（粗體，SGR 1）、`Dim`（暗色，SGR 2）、`Italic`（斜體，SGR 3）、
`Underlined`（底線，SGR 4）、`SlowBlink`（慢閃，SGR 5）、`RapidBlink`（快閃，SGR 6）、
`Reverse`（反轉，SGR 7）、`Hidden`（隱藏，SGR 8）、`CrossedOut`（刪除線，SGR 9）等。
提供對應的 `SetAttribute` Command，支援屬性的設定、取消與重置。
