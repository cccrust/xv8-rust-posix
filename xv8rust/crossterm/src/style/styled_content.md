# StyledContent 結構

`StyledContent` 是樣式設定的輸出封裝器。當 `Stylize` trait 的方法
（如 `.red()`、`.bold()`）被呼叫時，回傳此結構體，其中包含原始內容
（`&dyn Display`）與套用的 `ContentStyle`。`StyledContent` 實作 `Display`，
在輸出時自動附加與移除 ANSI SGR 逸出碼。此設計實現了零成本抽象——
樣式資訊僅在格式化階段消耗，不影響內容的原始型別。
