# mktemp — 建立暫存檔案或目錄

`mktemp` 建立唯一的暫存檔案或目錄。

## 核心設計

```rust
for _attempt in 0..1000 {
    let suffix: String = (0..6).map(|_| {
        let c = b"abcdefghijklmnopqrstuvwxyz0123456789"
            [rand() as usize % 36];
        c as char
    }).collect();
    let name = format!("/tmp/{}{}", prefix, suffix);
    // 嘗試建立
}
```

亂數產生 6 個字元的後綴，確保唯一性。

## 亂數產生

```rust
fn rand() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let mut seed = nanos.wrapping_mul(1103515245).wrapping_add(12345);
    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    seed
}
```

使用線性同餘生成器（LCG）產生偽隨機數。

## 檔案建立

```rust
if dir {
    std::fs::create_dir(&name)  // 目錄
} else {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&name)  // 檔案（create_new 確保不覆蓋）
}
```

`create_new(true)` 確保原子性：如果檔案已存在則失敗。

## 範本字串

```rust
let prefix = if template.contains('X') {
    &template[..template.find('X').unwrap()]
} else {
    template
};
```

`XXXXXX` 會被隨機字元替換：
- `tmp.XXXXXX` → `tmp.a3f7k2`

## 典型用途

### 安全暫存檔
```bash
# 建立暫存檔
tmpfile=$(mktemp)
echo "data" > "$tmpfile"
# ... 使用 ...
rm "$tmpfile"
```

### 暫存目錄
```bash
tmpdir=$(mktemp -d)
cd "$tmpdir"
# ... 工作 ...
cd / && rm -rf "$tmpdir"
```

### 批次處理
```bash
for i in {1..10}; do
    tmp=$(mktemp)
    process "$i" > "$tmp"
    # ...
done
```

## 失敗處理

```bash
tmp=$(mktemp) || exit 1
```

如果所有嘗試都失敗（機率極低），mktemp 會輸出錯誤並返回非零。

## 安全性

### 競態條件
`mktemp` 使用 `O_EXCL` 旗標確保原子性：
- 檢查和建立之間沒有時間窗口
- 其他程式無法搶先建立同名檔案

### 權限
```bash
# 安全的權限：只有擁有者可讀寫
tmp=$(mktemp)
ls -l "$tmp"
# -rw------- 1 user user 0 Jun  3 10:00 /tmp/tmp.abc123
```

## 選項

- `-d`：建立目錄
- `-t`：使用 `TMPDIR` 環境變數
- `-p DIR`：指定前綴目錄

## 輸出

成功時輸出完整路徑：
```
/tmp/tmp.abc123
```

## 與其他工具的比較

| 工具 | 用途 |
|------|------|
| `mktemp` | 安全建立唯一檔案 |
| `/tmp/file` | 可能被覆蓋/攻擊 |
| `tempfile` | 較不安全 |

## 底層系統呼叫

`mktemp` 使用：
- `open(name, O_CREAT | O_EXCL)`：原子建立
- `mkdir()`：目錄

## 實用範例

```bash
# 基本用法
mktemp

# 自訂前綴
mktemp myapp.XXXXXX

# 建立目錄
mktemp -d

# 在腳本中使用
#!/bin/bash
tmp=$(mktemp)
trap "rm -f $tmp" EXIT
```

## race condition 防護

`mktemp` 的 `O_EXCL` 確保：
1. 檢查檔案是否存在
2. 建立檔案

兩步驟是原子的，避免 TOCTOU（Time-of-check to time-of-use）問題。

## 相關指令

- `tmpfile`：glibc 的不安全替代
- `mkdir`：建立目錄
- `trap`：確保清理