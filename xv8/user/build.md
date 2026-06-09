# xv8 使用者程式建置腳本

`build.rs` 透過 `cargo::rustc-link-arg-bins` 傳遞 `user.ld` 連結器腳本給所有使用者空間二進位檔。
`user.ld` 定義使用者程式的 ELF 佈局：程式碼起始於 `0x00000000`，
設定 `.text`、`.rodata`、`.data`、`.bss` 段的相對位置，並確保堆疊邊界對齊。
使用者程式的載入位址由核心的 `exec()` 系統呼叫解析此佈局來載入記憶體。
