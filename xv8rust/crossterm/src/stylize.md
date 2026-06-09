# Stylize trait

提供對任何實作 `Display` 的型別進行內聯樣式設定的能力。
透過 `Stylize` trait，開發者可以直接對字串或數字呼叫 `.red()`、`.bold()`、
`.on_blue()` 等方法，而不需透過 `ContentStyle` 中介。此 trait 的設計
借鑑了 CSS 的鏈式樣式語法，回傳 `StyledContent` 封裝器，
在顯示時自動套用 ANSI SGR 逸出碼，實現流暢的 API 風格。
