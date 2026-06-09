# xv8-user-std 建置腳本

`build.rs` 取得 `CARGO_MANIFEST_DIR` 環境變數，建構 `user.ld` 的絕對路徑，
並透過 `cargo::rustc-link-arg` 傳遞給連結器。此連結器腳本定義了 xv8 Rust
標準函式庫覆蓋層（std overlay）的記憶體佈局，確保與核心載入器相容。
與核心使用者程式共用相同的 `user.ld`，使二進位檔可在 xv8 QEMU 中執行。
