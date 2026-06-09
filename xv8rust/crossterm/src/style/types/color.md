# Color 列舉

定義終端顏色值的列舉，涵蓋三種顏色深度：
**基礎色**（16 色：Black、Red、Green、Yellow、Blue、Magenta、Cyan、White 及其亮色版本）、
**8 位色**（`AnsiValue(u8)`，0–255 的 256 色調色板）、
**24 位真彩色**（`Rgb { r, g, b }`，支援 16 百萬色）。
`Display` 實作會將顏色轉換為對應的 ANSI SGR 序列參數
（前景色 `38;5;n` / `38;2;r;g;b`，背景色 `48;5;n` / `48;2;r;g;b`）。
