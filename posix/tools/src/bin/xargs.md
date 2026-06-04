# xargs — 將輸入轉換為命令參數

`xargs` 從標準輸入讀取項目，並將它們作為參數傳給指定命令。

## 核心設計

`xargs` 的核心是建立子程序並傳遞參數：

```rust
for item in &items {
    let mut child = Command::new(cmd_name);
    child.args(&base_args).arg(item)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = child.status()?;
}
```

## 輸入處理

`xargs` 從 stdin 讀取並拆分為單詞：

```rust
for line in stdin.lock().lines() {
    let line = line?;
    let trimmed = line.trim().to_string();
    if trimmed.is_empty() { continue; }
    for word in trimmed.split_whitespace() {
        items.push(word.to_string());
    }
}
```

注意：預設按空白分割，可以配合 `-0` 處理特殊字元。

## 參數替換（-I 選項）

`-I` 允許在命令中使用替換字串：

```rust
if let Some(ref repl) = replace_str {
    for item in &items {
        let mut child_args: Vec<String> = Vec::new();
        for a in &base_args {
            child_args.push(a.replace(repl, item));
        }
        // 執行 child_args
    }
}
```

範例：
```bash
find . -name "*.c" | xargs grep "TODO"   # 基本用法
echo "file1 file2" | xargs -I {} cp {} backup/  # -I 替換
```

## 批次大小（-n 選項）

`-n` 指定每次呼叫命令傳遞的參數數量：

```rust
if max_args > 0 {
    for chunk in items.chunks(max_args) {
        // 每 chunk 個項目執行一次
    }
}
```

範例：
```bash
echo "1 2 3 4 5" | xargs -n 2 echo
# 輸出：
# 1 2
# 3 4
# 5
```

## 預設命令

如果沒有指定命令，預設使用 `echo`：

```rust
let (cmd_name, base_args): (&str, Vec<&str>) = if cmd_args.is_empty() {
    ("echo", vec![])
} else {
    (cmd_args[0], cmd_args[1..].to_vec())
};
```

## 與 find -exec 的比較

```bash
# 使用 xargs（推薦）
find . -name "*.tmp" | xargs rm

# 使用 find -exec（每個檔案一個程序）
find . -name "*.tmp" -exec rm {} \;

# 使用 find -exec +（批次執行）
find . -name "*.tmp" -exec rm {} +
```

`xargs` 通常比分開執行更高效。

## 特殊字元處理

`xargs` 預設按空白分割。處理包含空白的檔案名：

```bash
# 使用 -0 處理 null 分隔的輸入
find . -name "*.c" -print0 | xargs -0 grep "TODO"

# 或配合 -I
find . -name "*.c" -exec grep -l TODO {} \;
```

## 退出行為

如果被呼叫的命令失敗，`xargs` 會終止並返回該 exit code：

```rust
if !status.success() {
    std::process::exit(status.code().unwrap_or(1));
}
```

使用 `-t` 查看實際執行的命令。

## 空輸入處理

```rust
if items.is_empty() { return; }
```

如果 stdin 為空，`xargs` 不執行任何命令（某些版本會執行一次 echo）。

## 典型用途

```bash
# 批量刪除
find . -name "*.bak" | xargs rm

# 批量重新命名
ls *.txt | xargs -I {} mv {} {}.old

# 批量壓縮
ls *.log | xargs gzip

# 找出的檔案交給 grep
find . -name "*.c" | xargs grep -l "TODO"
```

## 效能考量

- **批次大小**：較大的 `-n` 值減少程序建立開銷
- **並列執行**：GNU xargs 的 `-P` 選項可並列執行
- **記憶體**：處理大量輸入時注意系統限制

## 安全性考量

使用 `xargs` 處理未經信任的輸入時要小心：

```bash
# 危險：檔案名可能包含特殊字元
find . -name "*.txt" | xargs rm

# 安全：使用 null 分隔和 -0
find . -name "*.txt" -print0 | xargs -0 rm
```

## 底層系統呼叫

`xargs` 的核心是建立新程序：
- `fork()`：建立子程序
- `execve()`：執行命令
- `wait()`：等待命令完成

Rust 的 `std::process::Command` 封裝了這些。

## 與 xargs 的變體

- GNU xargs：完整功能
- BSD xargs：略有差異
- `parallel`：GNU parallel 更強大（支援並列執行）

## 相關指令

- `xargs`：將 stdin 轉換為命令參數
- `find -exec`：在 find 中直接執行命令
- `parallel`：並行執行