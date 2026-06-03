# mv — 移動或重新命名檔案

`mv`（move）用於移動檔案或目錄到新位置，或將其重新命名。

## 核心設計

`mv` 的實現相當簡潔，因為 Rust 的標準備函式庫提供了 `fs::rename`：

```rust
if let Err(e) = fs::rename(src, &dst) {
    // 跨設備移動：copy + remove
    if src.is_dir() {
        eprintln!("mv: cannot move '{}': {}", src.display(), e);
        std::process::exit(1);
    }
    if let Err(e2) = fs::copy(src, &dst) {
        eprintln!("mv: cannot copy '{}': {}", src.display(), e2);
        std::process::exit(1);
    }
    if let Err(e2) = fs::remove_file(src) {
        eprintln!("mv: cannot remove '{}': {}", src.display(), e2);
        std::process::exit(1);
    }
}
```

## 同一檔案系統的移動

在 POSIX 中，`rename` 是一個 atomic 操作：

```rust
fs::rename(src, &dst)
```

它的行為定義：
- 如果目標不存在：atomically 將 src rename 為 dst
- 如果目標存在且是空目錄：移除目標，將 src rename 為 dst
- 如果目標存在且非空：行為未定義（通常會失敗）

`rename` 的重要特性：
- **原子性**：操作要么完全成功，要么完全失敗
- **在同一檔案系統內**：不需要實際複製資料

## 跨檔案系統移動

當 `rename` 失敗（通常是 EXDEV, cross-device error）時：

```rust
if let Err(e) = fs::rename(src, &dst) {
    // 跨設備移動：copy + remove
    if src.is_dir() {
        eprintln!("mv: cannot move '{}': {}", src.display(), e);
        std::process::exit(1);
    }
    if let Err(e2) = fs::copy(src, &dst) {
        eprintln!("mv: cannot copy '{}': {}", src.display(), e2);
        std::process::exit(1);
    }
    if let Err(e2) = fs::remove_file(src) {
        eprintln!("mv: cannot remove '{}': {}", src.display(), e2);
        std::process::exit(1);
    }
}
```

注意：目錄跨設備移動會失敗（因為需要遞迴複製，且目錄結構可能改變）。

## 選項處理

```rust
let mut interactive = false;  // -i：覆寫前詢問
let mut force = false;         // -f：強制覆寫，不詢問
```

## 目標路徑決定

與 `cp` 類似：

```rust
let target = Path::new(&targets[targets.len() - 1]);
let sources = &targets[..targets.len() - 1];
let target_is_dir = target.is_dir();

let dst = if target_is_dir {
    target.join(src.file_name().unwrap_or_default())
} else {
    target.to_path_buf()
};
```

## 互動式確認

```rust
if dst.exists() && !force {
    if interactive {
        eprint!("mv: overwrite '{}'? ", dst.display());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if !input.trim().eq_ignore_ascii_case("y") { continue; }
    }
}
```

## 與 rm + cp 的差異

`mv` 比 `rm + cp` 的優勢：
1. **效能**：同檔案系統內是低成本操作，不複製資料
2. **原子性**：操作結果明確
3. **保留屬性**：通常保留時間戳、許可權等（取決於實作）

## inode 變化

- **同檔案系統**：`mv` 後 inode 不變（只是目錄條目改變）
- **跨檔案系統**：`mv` 後 inode 改變（等於建立新檔案）

## 底層系統呼叫

- `rename(oldpath, newpath)`：標準的重新命名系統呼叫
- `open/read/write/close`：當 rename 失敗時用於 copy+remove
- `unlink(path)`：刪除源檔案

## 安全性考量

如果目標檔案已存在，`mv` 的行為取決於作業系統和檔案系統：
- 某些系統會直接覆蓋
- 某些系統會失敗直到明確指定 `-f`

`mv` 應該確保目標的 meta-data（如 extended attributes）在何時被保留/丟失有明確定義。

## 與其他系統的差異

GNU coreutils 的 `mv` 實作更複雜：
- 支援 `--update` 選項，只在源比目標新時才移動
- 支援 `--no-target-directory` 防止目標目錄被錯誤使用
- 更好的 cross-device 處理

## 相關指令

- `cp`：複製檔案
- `rename`：批次重新命名
- `mmv`： Pattern-based 移動