# xv8 核心建置腳本

`build.rs` 透過 `cargo::rustc-link-arg-bin` 將 `kernel.ld` 連結器腳本傳遞給 `xv8` 二進位檔。
`kernel.ld` 定義核心 ELF 的記憶體佈局：起始地址為 `0x80000000`（QEMU virt 平台的 RAM 基底）、
程式碼段（`.text`）、唯讀資料（`.rodata`）、資料段（`.data`）、BSS 段的放置順序，
以及核心堆疊與中斷向量表的對齊方式。此腳本確保核心在機器模式啟動後能正確跳轉到 supervisor 模式。
