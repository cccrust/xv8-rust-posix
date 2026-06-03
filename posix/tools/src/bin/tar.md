# tar — 磁帶歸檔

`tar`（tape archive）將多個檔案合併為單一歸檔檔。

## 核心設計

### 建立歸檔

```rust
// Write POSIX tar header (simplified)
let mut header = [0u8; 512];
let name_bytes = fname.as_bytes();
let len = name_bytes.len().min(100);
header[..len].copy_from_slice(&name_bytes[..len]);

// Write size in octal
let size_str = format!("{:011o}", data.len());
let size_bytes = size_str.as_bytes();
let slen = size_bytes.len().min(12);
header[124..124+slen].copy_from_slice(&size_bytes[..slen]);
```

每個檔案以 512 位元組 header 開頭。

## TAR 格式

```
┌────────────────────────────────────┐
│  Header (512 bytes)                │
│  ├─ name[100]: 檔案名              │
│  ├─ mode[8]: 權限                   │
│  ├─ uid[8]: 使用者ID                │
│  ├─ gid[8]: 群組ID                 │
│  ├─ size[12]: 大小（八進位）        │
│  ├─ mtime[12]: 修改時間             │
│  ├─ chksum[8]: 標頭校驗             │
│  └─ ...                            │
├────────────────────────────────────┤
│  Data (512-byte blocks)            │
├────────────────────────────────────┤
│  ... (next file)                    │
├────────────────────────────────────┤
│  Two zero blocks (512 × 2)         │
└────────────────────────────────────┘
```

## 標頭解析

```rust
fn parse_tar_size(header: &[u8]) -> usize {
    let raw = std::str::from_utf8(&header[124..136]).unwrap_or("0");
    let trimmed = raw.trim_end_matches(|c| c == '\0' || c == ' ');
    usize::from_str_radix(trimmed, 8).unwrap_or(0)
}
```

size 欄位以八進位編碼。

## 讀取歸檔

```rust
while pos + 512 <= data.len() {
    let header = &data[pos..pos+512];
    if header.iter().all(|&b| b == 0) { break; }  // 結束標記

    let size = parse_tar_size(header);
    pos += 512;
    let file_data = &data[pos..pos+size];

    std::fs::write(name, file_data).unwrap();
    pos += ((size + 511) / 512) * 512;  // 對齊 512
}
```

## 區塊對齊

```rust
let rem = (512 - (data.len() % 512)) % 512;
if rem > 0 {
    out.write_all(&vec![0u8; rem]).unwrap();
}
```

資料填充到 512 位元組邊界。

## 結尾標記

```rust
// Two zero blocks at end
out.write_all(&[0u8; 1024]).unwrap();
```

歸檔結尾有兩個全零區塊。

## 操作模式

```rust
match c {
    'c' => create = true,   // 建立
    'x' => extract = true,  // 解開
    't' => list = true,     // 列表
}
```

## 典型用途

### 建立歸檔
```bash
tar -cf archive.tar dir/
```

### 解開歸檔
```bash
tar -xf archive.tar
```

### 列出內容
```bash
tar -tf archive.tar
```

## 壓縮

`tar` 本身不壓縮，但可結合：
```bash
tar -czf archive.tar.gz dir/    # gzip
tar -cjf archive.tar.bz2 dir/   # bzip2
tar -cJf archive.tar.xz dir/     # xz
```

## 選項

| 選項 | 意義 |
|------|------|
| `-c` | 建立歸檔 |
| `-x` | 解開歸檔 |
| `-t` | 列表 |
| `-f` | 檔案名 |
| `-v` | 詳細輸出 |

## 輸出格式

```bash
$ tar -cf archive.tar file1.txt file2.txt
$ tar -tf archive.tar
file1.txt
file2.txt
```

## 與 ZIP 的比較

| 特性 | tar | zip |
|------|-----|-----|
| 壓縮 | 無（需另加） | 內建 |
| 格式 | 流式 | 中央目錄 |
| 起源 | 1979 POSIX | 1989 PKZIP |

## 底層系統呼叫

`tar` 使用：
- `read()`：讀取原始檔案
- `write()`：寫入歸檔
- `stat()`：取得檔案屬性

## 實用範例

```bash
# 建立
tar -cf app.tar src/ bin/

# 解開
tar -xf app.tar -C /opt/

# 追加（不支援）
# tar -rf 不在此實現

# 查看
tar -tvf app.tar
```

## 安全性

`tar` 可能建立任意路徑的檔案（路徑穿越）：
```bash
# 惡意歸檔
tar -cf evil.tar ../../etc/passwd
```

解開時應注意。

## 相關指令

- `zip`：壓縮歸檔
- `gzip`/`gunzip`：壓縮
- `cpio`：另一種歸檔格式