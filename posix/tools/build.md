# POSIX 工具建置腳本

僅在目標架構為 RISC-V（`riscv64`）時啟用，透過 `cargo::rustc-link-arg-bins`
傳遞 `user.ld` 給所有 POSIX 工具二進位檔。在主機（x86_64/aarch64）編譯時，
`build.rs` 不做任何特殊處理，使用標準系統 `ld` 佈局。此設計讓同一套原始碼
可同時編譯為原生 POSIX 工具與 xv8 核心上的使用者程式，實現跨平台一致性。
