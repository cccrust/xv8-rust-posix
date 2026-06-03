# touch — 變更檔案時間戳或建立檔案

`touch` 有兩個主要功能：
1. 將檔案的存取和修改時間更新為現在
2. 如果檔案不存在，則建立一個空檔案

## 時間戳類型

每個檔案有三個時間戳：
- **atime（access time）**：最後讀取時間
- **mtime（modification time）**：最後修改內容時間
- **ctime（change time）**：inode 最後變更時間（無法直接設定）

`touch` 主要修改前兩個。

## 核心實作

```rust
if !exists {
    if no_create { continue; }
    fs::write(path, "")?;  // 建立空檔案
}
// 設定時間戳
libc::utimes(path_c.as_ptr(), [atime, mtime].as_ptr());
```

`utimes` 是設定時間戳的系統呼叫，接受兩個 `timeval` 結構（秒+微秒）。

## 時間解析

`touch` 支援多種時間格式：

### MMDDhhmm[.ss]（預設格式）
8個字元，例如 `12252345` = 12月25日 23:45，使用目前年份。

### YYMMDDhhmm（兩位年份）
10個字元。

### YYYYMMDDhhmm（四位年份）
12個字元。

```rust
let len = datetime.len();
if len == 8 {
    // MMDDhhmm
    let month: u32 = datetime[0..2].parse()?;
    // ...
} else if len == 10 {
    // YYMMDDhhmm
    // ...
} else if len == 12 {
    // YYYYMMDDhhmm
    // ...
}
```

## 閏年處理

時間計算需要正確處理閏年：

```rust
let days_in_year = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
```

每四年一閏，百年不閏，四百年又閏。

## 選項處理

```rust
let mut no_create = false;      // -c：不建立檔案
let mut ref_file: Option<String> = None;  // -r：使用參考檔案的時間
let mut time_str_opt: Option<String> = None;  // -t：指定時間
let mut set_access = false;     // -a：只設定 atime
let mut set_modify = false;     // -m：只設定 mtime
```

- `-c`：如果檔案不存在，不建立它
- `-r`：使用參考檔案的時間
- `-t`：使用指定時間（格式與 -d 不同）
- `-a` 和 `-m`：選擇性設定時間

## 時間的預設行為

如果既沒有 `-a` 也沒有 `-m`，兩者都設定（這是 POSIX 行為）。

如果都沒有指定任何時間相關選項，使用目前時間。

## 底層系統呼叫

- `utimes(path, times)`：設定檔案的 access 和 modification time
- `utimensat(dirfd, path, times, AT_SYMLINK_NOFOLLOW)`：更精確的版本（納秒精度）
- `stat(path, buf)`：讀取目前時間戳

## 典型用途

1. **建立空檔案**：`touch newfile`
2. **更新時間戳**：`touch existingfile`
3. **批量建立檔案**：`touch file{1..10}.txt`
4. **避免編譯器重新編譯**：`touch configure.ac` 強制 reconfigure

## 與其他工具的差異

- `mkdir`：只建立目錄，不改變時間
- `cp -p`：複製並保留時間
- `install`：可以設定任意時間（但非標準）

## 時區和夏令時

`touch` 處理的時間是本地時間（使用系統時區）。在跨系統傳輸時需要注意時區轉換。

## 相關指令

- `stat`：顯示檔案詳細時間戳
- `ls -l`：顯示 mtime
- `ls -lu`：顯示 atime
- `ls -lc`：顯示 ctime