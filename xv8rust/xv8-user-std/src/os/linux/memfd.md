# Linux memfd 包裝器

提供 `memfd_create()` 系統呼叫的 Rust 包裝。memfd 建立在核心記憶體中的匿名檔案，
返回檔案描述子，特點是完全在 RAM 中運作（掛載於 `tmpfs` 的匿名 inode）。
支援 `MFD_CLOEXEC`（執行時關閉）、`MFD_ALLOW_SEALING`（允許不可變封印）、
`MFD_HUGETLB`（使用巨型頁面）等旗標。memfd 常見用途：共享記憶體區段、
安全地建立暫存檔案（不佔用磁碟空間）、seccomp 不可變封印保護。
在 xv8 中作為 `memfd` 與 `memfd_create` 系統呼叫的介面。
