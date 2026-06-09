# Colored 包裝器

`Colored` 是對任何型別 `T` 的包裝結構，儲存 `T` 的值與一組 `Colors`。
當 `Display` 被呼叫時，自動在內容前後套用 ANSI 顏色逸出碼。
此結構主要作為 `Stylize` trait 鏈式操作的內部表示，
讓開發者可以 `.red().on_blue()` 連續呼叫，最終產出 `StyledContent`。
