# cp — 複製檔案與目錄

`cp`（copy）用於複製檔案和目錄，是檔案操作中最基本也最常用的命令之一。

## 設計結構

`cp` 的實作分為兩個主要函式：
- `copy_file`：複製單個檔案
- `copy_dir`：遞迴複製目錄

```rust
fn copy_file(src: &Path, dst: &Path, preserve: bool) -> Result<(), String> {
    fs::copy(src, dst).map_err(|e| format!("{}", e))?;
    if preserve {
        if let Ok(meta) = fs::metadata(src) {
            let _ = fs::set_permissions(dst, meta.permissions());
        }
    }
    Ok(())
}
```

## 檔案複製機制

Rust 的 `fs::copy` 底層會：
1. 以唯讀模式開啟源檔案
2. 建立目標檔案
3. 迴圈讀取源檔案內容並寫入目標

```rust
// 等價於以下流程：
let mut reader = File::open(src)?;
let mut writer = File::create(dst)?;
io::copy(&mut reader, &mut writer)?;
```

## 目錄遞迴複製

```rust
fn copy_dir(src: &Path, dst: &Path, recursive: bool, preserve: bool, interactive: bool, force: bool) -> Result<(), String> {
    if !recursive {
        return Err(format!("omitting directory '{}'", src.display()));
    }
    fs::create_dir_all(dst)?;
    let entries = fs::read_dir(src)?;
    for entry in entries {
        // 遞迴處理每個項目
    }
}
```

遞迴時會保持目錄結構，對每個子目錄遞迴呼叫 `copy_dir`。

## 選項解析

```rust
let mut recursive = false;   // -R, -r：遞迴複製目錄
let mut preserve = false;    // -p：保留時間戳和許可權
let mut interactive = false;  // -i：覆寫前詢問
let mut force = false;       // -f：強制覆寫
```

## 目標路徑推斷

當有多個源檔案時，目標必須是目錄：

```rust
let target = Path::new(&srcs[srcs.len() - 1]);
let sources = &srcs[..srcs.len() - 1];
let target_is_dir = target.is_dir();

if sources.len() > 1 && !target_is_dir {
    eprintln!("cp: target '{}' is not a directory", target.display());
    std::process::exit(1);
}
```

如果是目錄，則每個源檔案複製到目標目錄下，保持原始檔案名：

```rust
let dst = if target_is_dir {
    target.join(src.file_name().unwrap_or_default())
} else {
    target.to_path_buf()
};
```

## 許可權保留

`-p` 選項保留原始檔案的許可權和時間戳：

```rust
if preserve {
    if let Ok(meta) = fs::metadata(src) {
        let _ = fs::set_permissions(dst, meta.permissions());
    }
}
```

注意：時間戳的保留在 xv8 上可能不完全支援。

## 互動式覆寫確認

```rust
if dst_path.exists() && !force {
    if interactive {
        eprint!("cp: overwrite '{}'? ", dst_path.display());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if !input.trim().eq_ignore_ascii_case("y") { continue; }
    }
}
```

## 錯誤處理策略

- 檔案存在且 `force` 未設定：報錯退出
- 目錄複製時無 `recursive` 選項：忽略並報錯
- 複製失敗：輸出錯誤並退出

```rust
if let Err(e) = copy_file(src, &dst, preserve) {
    eprintln!("cp: cannot copy '{}': {}", src.display(), e);
    std::process::exit(1);
}
```

## Copy-on-Write 考量

雖然 `cp` 本身不涉及 Copy-on-Write（這是核心 fork 的功能），但在某些優化實現中，大檔案的複製可以採用類似 COW 的機制：只有當任一副本被寫入時才真正複製資料。

## 底層系統呼叫

`cp` 依賴的 syscall：

- `open(path, O_RDONLY)`：開啟源檔案
- `create(path, mode)`：建立目標檔案
- `read(fd, buf, n)`：讀取資料
- `write(fd, buf, n)`：寫入資料
- `close(fd)`：關閉檔案描述符
- `mkdir(path, mode)`：建立目錄
- `chmod(path, mode)`：設定許可權

## 效能瓶頸

- 大檔案：使用較大的緩衝區減少系統呼叫
- 大量小檔案：每個檔案都需要獨立的 open/read/write/close 操作
- 跨檔案系統複製：需要讀取和寫入，無法使用 rename 的低成本移動

## 相關指令

- `mv`：移動/重新命名檔案
- `scp`/`rsync`：網路複製
- `cpio`：封存複製