# ABI — 核心與使用者空間的介面契約

## 概述

ABI（Application Binary Interface）定義了 xv8 核心與使用者程式之間的二進制通訊協定。不同於 API（原始碼層級），ABI 規範的是記憶體佈局、系統呼叫編號、資料結構的位元組排列等執行期約定。

## 核心資料結構

### Stat
`Stat` 是 `stat` 系統呼叫回傳的檔案中繼資料結構，包含檔案類型（一般檔案、目錄、裝置）、大小、inode 編號、連結數、權限位元及三個時間戳（atime/mtime/ctime）。其佈局對應 Linux 的 `struct stat`，確保與 POSIX 使用者空間相容。

### Directory
`Directory` 是目錄讀取操作的 entry 結構，每個 entry 包含 inode 編號、檔案名稱長度與檔案名稱。核心在 `readdir` 系統呼叫中依序填入此結構，使用者程式透過 getdents 取得目錄清單。

### Syscall
系統呼叫編號的列舉型別，每個 variant 對應一個整數常數。x86 Linux 使用 `rax` 傳遞編號，RISC-V 則透過 `a7` 暫存器。`Syscall` 的定義必須與核心的 `syscall_handler` 分發表完全一致，否則會造成呼叫錯位。

## 設計原則

ABI 的穩定性是作業系統的基石——一旦釋出，使用者程式即依賴於此二進制介面。xv8 在開發階段允許變動，但鎖定後的所有變更必須向後相容。

## 相關文件

- [syscall.md](./syscall.md) — 系統呼叫處理流程
- [trap.md](./trap.md) — 使用者/核心模式切換
- [sysfile.md](./sysfile.md) — 檔案系統系統呼叫
- [sysproc.md](./sysproc.md) — 行程管理系統呼叫
