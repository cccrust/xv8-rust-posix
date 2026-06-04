# ls — 列出目錄內容

`ls`（list）是 Unix 系統中最常用的命令之一，用於列出目錄中的檔案和子目錄資訊。

## 核心功能

`ls` 的複雜度在於它需要處理多種輸出格式和顯示選項。核心功能包括：

1. 讀取目錄項目（readdir）
2. 獲取每個項目的元資料（stat/lstat）
3. 格式化並輸出結果

## 許可權字串解析

`ls` 最複雜的部分之一是將數值許可權模式轉換為符號表示。`mode_string` 函式展示了這個轉換過程：

```rust
fn mode_string(mode: u32) -> String {
    let t = match mode & 0o170000 {
        0o100000 => '-',  // 普通檔案
        0o040000 => 'd',  // 目錄
        0o120000 => 'l',  // 符號連結
        0o020000 => 'c',  // 字元裝置
        0o060000 => 'b',  // 區塊裝置
        0o010000 => 'p',  // FIFO
        0o140000 => 's',  // 插槽
        _ => '?',
    };
    // ...
}
```

許可權模式使用 4 位八進位數：
- 第一位：檔案類型（0o170000 是遮罩）
- 其餘三位：owner/group/other 的 rwx 許可權

## Setuid/Setgid/Sticky Bit

除了基本的 rwx 許可權，還有特殊的 setuid（`s`）、setgid（`s`）和 sticky（`t`）位：

```rust
match (mode & 0o100 != 0, mode & 0o4000 != 0) {
    (true, true) => 's',   // setuid 且有執行權限
    (false, true) => 'S',  // setuid 但無執行權限
    (true, false) => 'x',  // 普通執行權限
    _ => '-',
}
```

## 使用者/群組名稱解析

嘗試將 UID/GID 解析為可讀的名稱：

```rust
fn user_name(uid: u32) -> String {
    #[cfg(unix)]
    unsafe {
        let pw = libc::getpwuid(uid);
        if !pw.is_null() {
            return std::ffi::CStr::from_ptr((*pw).pw_name).to_string_lossy().to_string();
        }
    }
    uid.to_string()  // 解析失敗時回退為數字
}
```

這調用 POSIX 的 `getpwuid()` 和 `getgrgid()` 函式。

## 時間格式

時間顯示有特殊邏輯：6 個月內的檔案顯示時間（時:分），之外的顯示年份：

```rust
let is_recent = (secs - now_secs).abs() < 180 * 86400;
if is_recent {
    format!("{} {:2} {:02}:{:02}", MONTHS[mo as usize], day, h, mi)
} else {
    format!("{} {:2}  {:4}", MONTHS[mo as usize], day, y)
}
```

`180 * 86400` = 180 天的秒數。

## 目錄讀取與排序

```rust
let entries = fs::read_dir(path)?;
let mut v: Vec<_> = r.filter_map(|e| e.ok()).collect();
v.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
```

使用 `read_dir` 讀取目錄，過濾錯誤項目，按檔案名排序。隱藏檔案（以 `.` 開頭）預設被過濾。

## 多欄位輸出計算

`ls -l` 輸出固定寬度格式。普通模式需要計算列數：

```rust
let max_width = names.iter().map(|n| n.len()).max().unwrap_or(0) + 2;
let term_width = 80;  // 預設終端寬度
let cols = std::cmp::max(1, term_width / max_width.max(1));
```

這決定了每行列數以及何時換行。

## 遞迴列表

`-R` 選項遞迴列出所有子目錄：

```rust
if cfg.recursive {
    for (name_bytes, _, is_dir) in &items {
        if *is_dir {
            // 遞迴呼叫 list_dir
            list_dir(&sub_path, sub_path.to_string_lossy().as_ref(), cfg, false, first);
        }
    }
}
```

## inode 和區塊計數

`-i` 顯示 inode 編號，`-s` 顯示區塊數（每區塊 512 位元組）：

```rust
let total: u64 = items.iter().map(|(_, m, _)| m.blocks()).sum();
println!("total {}", total);
```

`blocks()` 回傳檔案佔用的作業系統區塊數。

## 分類標記

`-F` 選項在可執行檔後加 `*`，目錄後加 `/`，符號連結後加 `@`：

```rust
if cfg.classify {
    let c = if *is_dir { '/' } else if meta.permissions().mode() & 0o111 != 0 { '*' } else { ' ' };
}
```

## 底層系統呼叫

`ls` 使用的主要 syscall：

- `getdents64(fd, buf, size)`：讀取目錄項目
- `stat(path, buf)`：獲取檔案狀態（追隨符號連結）
- `lstat(path, buf)`：獲取檔案狀態（不追隨符號連結）
- `access(path, mode)`：檢查檔案存取許可權

## 符號連結處理

使用 `symlink_metadata` 而非 `metadata`，避免追隨符號連結：

```rust
match fs::symlink_metadata(path) {
    Ok(meta) => { /* ... */ }
}
```

這確保 `ls -l` 顯示符號連結本身的資訊（如 `link -> target`），而非目標檔案的資訊。

## 相關指令

- `tree`：以樹狀結構顯示目錄
- `find`：搜尋並列出檔案
- `stat`：顯示檔案詳細資訊