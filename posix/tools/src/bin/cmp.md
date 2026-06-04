# cmp — 位元組級檔案比較

`cmp` 比較兩個檔案，如果相同則不輸出，否則報告第一個不同的位元組位置。

## 核心設計

```rust
loop {
    let n1 = f1.read(&mut buf1).unwrap_or(0);
    let n2 = f2.read(&mut buf2).unwrap_or(0);

    if n1 == 0 && n2 == 0 {
        return; // 檔案相同
    }

    if n1 != n2 || buf1[..n1.min(n2)] != buf2[..n1.min(n2)] {
        for j in 0..n1.min(n2) {
            if buf1[j] != buf2[j] {
                println!("{} {} {:o} {:o}",
                    path1.display(), offset + j as u64, buf1[j], buf2[j]);
                std::process::exit(1);
            }
        }
        // 長度不同導致不同
        println!("cmp: EOF on {}", if n1 < n2 { path1.display() } else { path2.display() });
        std::process::exit(1);
    }

    offset += n1 as u64;
}
```

`cmp` 以區塊讀取兩個檔案，逐位元組比較。

## 輸出格式

當發現差異時：
```
file1 file2 位置 八進位(檔案1) 八進位(檔案2)
```

例如：
```
file1.txt file2.txt 5 012 061
```

表示：
- 第 5 位元組處有差異
- `file1.txt` 該位置是 `\n`（八進位 012）
- `file2.txt` 該位置是 `9`（八進位 061，即 `9` 的 ASCII）

## 選項處理

```rust
let mut silent = false;  // -s：安靜模式（不輸出）
```

`-s`（silent）選項讓 `cmp` 在發現差異時不回傳任何訊息，只返回非零退出碼。

## 退出碼

- `0`：檔案相同
- `1`：檔案不同
- `2`：錯誤（檔案不可讀取等）

```rust
if !silent {
    println!("...");
}
std::process::exit(1);
```

## 與 diff 的比較

| 特性 | `cmp` | `diff` |
|------|-------|--------|
| 比較層級 | 位元組 | 行 |
| 速度 | 快（遇差即停） | 慢 |
| 輸出 | 位置和差異位元組 | 行級差異 |
| 用途 | 二進制比較 | 文字差異 |

## 典型用途

### 檢查檔案是否相同
```bash
if cmp -s file1.txt file2.txt; then
    echo "Files are identical"
fi
```

### 二進制檔案比較
```bash
cmp image1.png image2.png
# 或
cmp -s image1.png image2.png && echo "Same" || echo "Different"
```

### 驗證下載完整性
```bash
cmp downloaded.iso original.iso
```

## 緩衝區大小

```rust
let mut buf1 = [0u8; 8192];
let mut buf2 = [0u8; 8192];
```

使用 8KB 緩衝區讀取，這是效能和記憶體的平衡。

## 偏移量計算

```rust
let mut offset: u64 = 1; // POSIX: 1-indexed
```

偏移量從 1 開始（POSIX 規範）。

## EOF 處理

當一個檔案比另一個短時：
```bash
echo "cmp: EOF on file2"
```

表示在短檔案結束後，長檔案還有內容。

## 與 checksum 的比較

| 工具 | 用途 | 輸出 |
|------|------|------|
| `cmp` | 找出第一個差異 | 位置和值 |
| `md5sum` | 驗證完整性 | 校驗和 |
| `diff` | 行級差異 | unified diff |

## 典型輸出

```bash
$ cmp file1 file2
file1 file2 10 0A 0D
```

第 10 位元組不同：`0A`（換行）vs `0D`（回車）。

## 底層系統呼叫

`cmp` 使用：
- `open()`：開啟檔案
- `read()`：讀取資料
- `close()`：關閉檔案

## 安全考量

`cmp` 只需讀取權限，不需要寫入。

## 實用範例

```bash
# 快速檢查
cmp -s program new_program && echo "Same" || echo "Different"

# 比較二進制檔案
cmp /bin/ls /backup/ls

# 配合 shell 條件
[ "$(cmp -s a b; echo $?)" -eq 0 ] && echo "Identical"
```

## 與 cmp 的變體

- `cmp -l`：顯示所有差異位元組
- `cmp -i`：跳過初始位元組

## 相關指令

- `diff`：行級比較
- `md5sum`：校驗和
- `sha256sum`：SHA256 校驗
- `fc`：DOS/Windows 的 `fc` 命令