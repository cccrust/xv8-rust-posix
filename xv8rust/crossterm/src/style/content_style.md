# ContentStyle 結構

定義完整文字樣式的資料結構，包含前景色、背景色、屬性集合與超連結 URL。
支援 Builder 風格的鏈式建構（`.fg(color)`、`.bg(color)`、`.bold()` 等），
以及樣式合併（`+` 運算子重載）。`ContentStyle` 可套用於任何實作 `Display` 的型別，
透過 ANSI SGR 序列包裹輸出內容。此設計讓樣式可被組合、重複使用與繼承。
