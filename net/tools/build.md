# 網路工具建置腳本

與 POSIX 工具的 `build.rs` 邏輯相同：僅在目標架構為 `riscv64` 時，
透過 `cargo::rustc-link-arg-bins` 傳遞 `user.ld` 給網路工具二進位檔。
`user.ld` 定義 RISC-V 使用者程式的記憶體佈局，使網路工具（ping、dns、tcp、curl 等）
可在 xv8 QEMU 的核心網路堆疊（E1000 + IPv4/UDP/DHCP）上運作。
主機編譯時不啟用特殊連結腳本，保留原生執行能力。
