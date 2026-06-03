# chmod — 變更檔案許可權模式

`chmod`（change mode）用於變更檔案或目錄的存取許可權。

## POSIX 許可權模型

每個檔案有三組許可權：
- **Owner（擁有者）**：檔案的擁有者
- **Group（群組）**：檔案所屬的群組
- **Other（其他）**：其他所有使用者

每組有三位許可權：
- **r（Read）**：讀取權限
- **w（Write）**：寫入權限
- **x（Execute）**：執行權限

此外還有特殊位：
- **setuid（s）**：執行時以檔案擁有者身份執行
- **setgid（s）**：執行時以檔案所屬群組身份執行
- **sticky（t）**：目錄中只有擁有者能刪除檔案

## 許可權的數值表示

許可權用 4 位八進位數表示：
```
0oABCD
││││└─ Other: r=4, w=2, x=1
│││└── Group
││└─── Owner
│└──── SUID/SGID/Sticky
└───── 檔案類型（0o7777 的高位）
```

## 數值模式

直接使用八進位數字指定許可權：

```rust
if s.as_bytes().first()?.is_ascii_digit() {
    return u32::from_str_radix(s, 8).ok();
}
```

例如 `chmod 755 file` 等於 `chmod u=rwx,go=rx file`。

## 符號模式解析

xv8 的 `chmod` 支援符號模式，格式為 `[who] [operator] [perm]`：

```rust
// 解析 who
let who_mask = if who.contains('u') { 0o7700 } else { 0 }
    | if who.contains('g') { 0o7070 } else { 0 }
    | if who.contains('o') { 0o7007 } else { 0 }
    | if who.is_empty() || who.contains('a') { 0o7777 } else { 0 };

// 解析 perm
let perm_bits = if perms.contains('r') { 0o444 } else { 0 }
    | if perms.contains('w') { 0o222 } else { 0 }
    | if perms.contains('x') { 0o111 } else { 0 }
    | if perms.contains('s') { 0o6000 } else { 0 }
    | if perms.contains('t') { 0o1000 } else { 0 };
```

操作符：
- `+`：添加指定的許可權
- `-`：移除指定的許可權
- `=`：設定為指定的許可權

例如：
- `u+rwx`：給擁有者添加讀、寫、執行
- `go-w`：從群組和其他移除寫入
- `a=rx`：設定所有人為讀取和執行

## 遞迴修改

`-R` 選項遞迴修改目錄中所有檔案：

```rust
fn apply_mode(path: &Path, mode: u32, recursive: bool) {
    if recursive && path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                apply_mode(&entry.path(), mode, true);
            }
        }
    }
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
}
```

## 底層系統呼叫

- `chmod(path, mode)`：改變檔案模式
- `fchmod(fd, mode)`：透過檔案描述符改變模式

注意：改變 setuid/setgid 位需要 root 許可權。

## 許可權與安全

setuid 的安全問題：
- 允許程式以其他使用者身份執行
- 錯誤使用可能導致權限提升
- 現代 Unix 系統常禁用 setuid

sticky bit 在 `/tmp` 目錄的用途：
- 防止使用者刪除他人的檔案
- 即使其他使用者有寫入權限

## 相關指令

- `chown`：改變擁有者
- `chgrp`：改變所屬群組
- `umask`：設定新建檔案的預設許可權遮罩