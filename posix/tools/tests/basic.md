# POSIX 工具基礎測試套件

此整合測試使用 Rust 的 `#[test]` 框架，透過 `Command` 產生實際的 POSIX 工具行程
來驗證正確性。測試分階段涵蓋：基本 I/O（true、false、echo、yes）、
檔案操作（mkdir、rmdir、ln、touch、chmod、ls、cp、mv、rm）、
文字處理（head、tail、sort、uniq、cut、tee、od、cmp、diff）、
搜尋與過濾（grep、sed、xargs）、系統工具（test、date、du、nice、ps）、
shell 行為（pipe、redirect、heredoc、and/or 運算子）、進階工具（find、comm、nl、join、paste、split），
以及後期加入的 printf、expr、file、pathchk、link、unlink、mkfifo 等。
測試設計為無狀態、可並行執行，使用臨時目錄避免互相干擾。
