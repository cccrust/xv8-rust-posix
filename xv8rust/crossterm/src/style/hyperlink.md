# 超連結支援

實作終端超連結（OSC 8 escape sequence）的標準。
OSC 8 序列格式為 `\x1b]8;params;uri\x1b\\...內容...\x1b]8;;\x1b\\`，
允許終端模擬器將文字渲染為可點擊的超連結。此模組提供 `SetHyperlink` Command，
支援設定與關閉超連結。OSC 8 已獲 xterm、kitty、iTerm2 等主流終端模擬器支援，
為終端應用帶來現代化的互動體驗。
