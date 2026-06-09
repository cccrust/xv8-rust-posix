# crossterm 巨集

提供三個主要巨集：`crossterm!`——終端操作的最簡潔語法糖；
`execute!`——立即執行一個或多個 `Command`，返回 `Result`；
`queue!`——將指令排入 `Write` 實作的緩衝區，可與 `flush()` 搭配批次執行。
這些巨集利用 `Command` trait 的統一介面，大幅減少重複的 ANSI 字串拼接與寫入程式碼。
