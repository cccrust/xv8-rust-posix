# rm — 移除檔案或目錄

`rm`（remove）用於刪除檔案或目錄。

## 設計結構

`rm` 的實作分為兩部分：
- `fs::remove_file`：刪除普通檔案
- `remove_dir_recursive`：遞迴刪除非空目錄

```rust
fn remove_dir_recursive(path: &Path) -> Result<(), String> {
    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let p = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            remove_dir_recursive(&p)?;
        } else {
            fs::remove_file(&p)?;
        }
    }
    fs::remove_dir(path)?;
    Ok(())
}
```

## 選項解析

```rust
let mut recursive = false;   // -r, -R：遞迴刪除目錄
let mut force = false;        // -f：強制刪除，不提示
let mut interactive = false;  // -i：刪除前詢問
```

## 選項行為

```rust
if !path.exists() {
    if force { continue; }  // -f 忽略不存在的檔案
    eprintln!("rm: cannot remove '{}': No such file or directory", path.display());
    std::process::exit(1);
}

if interactive {
    // 詢問確認
}

if path.is_dir() {
    if !recursive {
        eprintln!("rm: cannot remove '{}': Is a directory", path.display());
    }
    // 遞迴刪除
} else {
    fs::remove_file(path)?;
}
```

## 目錄刪除的挑戰

刪除非空目錄需要先刪除所有內容。遞迴演算法：
1. 讀取目錄中的所有項目
2. 對每個子目錄遞迴呼叫刪除函式
3. 對每個普通檔案呼叫 `remove_file`
4. 刪除空目錄本身

這意味著需要多次 `readdir` 和謹慎的錯誤處理。

## 符號連結的處理

`rm` 刪除符號連結，而非符號連結指向的目標：

```rust
fs::remove_file(&p)  // 這會刪除連結本身
```

即使符號連結指向不存在的目標（斷裂連結），`rm` 仍然可以刪除它。

## 許可權檢查

刪除檔案不需要對檔案本身有許可權，但需要對**父目錄**有寫入和執行許可權（`w` 和 `x`）。

## 硬連結與符號連結

- **硬連結**：當檔案有多個連結時，`rm` 只減少連結計數（nlink），只有 nlink 為 0 時才釋放磁碟空間
- **符號連結**：`rm` 刪除連結，不影響原始檔案

## -force 選項的行為

`-f` 選項的精確行為：
- 不存在的檔案不報錯（靜默忽略）
- 不提示確認（跳過 `-i`）
- 不顯示錯誤（但仍返回非零 exit code）

```rust
if !path.exists() {
    if force { continue; }  // -f 靜默忽略
}
```

## 底層系統呼叫

- `unlink(path)`：刪除檔案的目錄條目
- `rmdir(path)`：刪除空目錄
- `getdents64(fd, buf, size)`：讀取目錄內容

## 危险操作與防範

`rm -rf /` 是最著名的危险命令之一：
- `-r`：遞迴刪除所有子目錄
- `-f`：強制刪除，不提示確認
- `/`：根目錄

現代系統對此有一定保護：
- 需要 root 許可權
- 某些發行版有「安全網」防止意外刪除

## 恢復已刪除檔案

一旦 `rm` 刪除了檔案，資料就從檔案系統中移除。要恢復需要：
- 從備份恢復
- 使用檔案系統恢復工具（在資料被覆寫前）

## 延遲刪除

某些系統支援「mv to trash」而非直接刪除，提供了一層保護。Linux 上的實現是 `trash-cli` 或 freedesktop.org 的 Trash 規格。

## 相關指令

- `unlink`：刪除單個檔案（較 low-level）
- `rmdir`：刪除空目錄
- `trash`：移動到回收筒