# Stylize trait（style 模組）

此模組包含 `Stylize` trait 的實際實作，而 `stylize.rs`（上層）作為 re-export。
定義了針對所有泛型型別 `T: Display` 的樣式方法預設實作，包括：
前景色 `.with()`/`.fg()`、背景色 `.on()`/`.bg()`、屬性 `.bold()`/`.italic()` 等。
採用 marker type 技巧將顏色轉換為 `Color` 型別，避免 trait 衝突，
確保 API 的型別安全性與表達力。
