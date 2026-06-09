# Command trait

定義 crossterm 的核心抽象——`Command` trait。實作此 trait 的型別代表一個可排入佇列的終端操作，
包含 `write_ansi`（產生 ANSI 逸出碼）與 `execute_winapi`（Windows API 實作）兩個方法。
`Command` 的設計讓操作可延遲執行：先收集一系列指令，再一次性寫入標準輸出，
減少核心態轉換次數。此模式類似命令模式（Command Pattern），支援 `queue!` 與 `execute!` 巨集。
