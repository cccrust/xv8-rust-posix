# df — 檔案系統磁碟空間

`df`（disk filesystem）顯示檔案系統的總空間、已用空間、可用空間。

## 核心設計

```rust
#[cfg(unix)]
{
    let mounts = Path::new("/proc/mounts");
    let _content = if mounts.exists() {
        std::fs::read_to_string(mounts).unwrap_or_default()
    } else {
        String::new()
    };

    let paths: Vec<&str> = if i < args.len() { vec![&args[i]] } else { vec!["/"] };
    for p in &paths {
        if let Ok(_stat) = std::fs::metadata(path) {
            println!("Filesystem     1M-blocks  Used Available Use% Mounted on");
            println!("{:<15}        -     -         -    - {}", "?", p);
        }
    }
}
```

xv8 的 `df` 是簡化版本，主要讀取 `/proc/mounts` 和嘗試 stat 檔案系統。

## 讀取掛載點

```rust
let mounts = Path::new("/proc/mounts");
if mounts.exists() {
    std::fs::read_to_string(mounts).unwrap_or_default()
}
```

`/proc/mounts`（Linux）或 `/etc/mtab`（傳統）包含所有掛載點。

## statfs 系統呼叫

完整的 `df` 使用 `statfs` 或 `statvfs` 系統呼叫：

```rust
struct statfs {
    f_type: u32,        // 檔案系統類型
    f_bsize: u64,       // 最優傳輸區塊大小
    f_blocks: u64,      // 總區塊數
    f_bfree: u64,       // 免費區塊數
    f_bavail: u64,      // 可用區塊數（非 root）
    f_files: u64,       // 總 inode 數
    f_ffree: u64,       // 免費 inode 數
}
```

## 輸出格式

```
Filesystem     1M-blocks  Used Available Use% Mounted on
/dev/sda1          51200  25600     24000  52% /
tmpfs               2048     64      1984   4% /tmp
```

欄位說明：
- **1M-blocks**：總空間（以 1MB 為單位）
- **Used**：已用空間
- **Available**：可用空間（非 root）
- **Use%**：使用百分比
- **Mounted on**：掛載點

## 選項處理

```rust
let mut human = false;  // -h：人類可讀格式

match c {
    'h' => human = true,
    _ => { eprintln!("df: invalid option -- '{}'", c); }
}
```

## 人類可讀格式（-h）

```rust
if human {
    // 使用 K、M、G 等單位顯示
}
```

與 `du` 的人類可讀邏輯類似。

## 與 du 的比較

| 特性 | `df` | `du` |
|------|------|------|
| 計算層面 | 檔案系統 | 實際檔案 |
| 速度 | 快（superblock） | 慢（掃描） |
| 用途 | 磁碟空間 | 目錄佔用 |

## 磁碟空間計算

```rust
let total = f_blocks * f_bsize;
let used = (f_blocks - f_bfree) * f_bsize;
let available = f_bavail * f_bsize;
let percent = (used as f64 / total as f64) * 100.0;
```

## 典型用途

```bash
# 查看所有掛載點
df -h

# 查看特定檔案系統
df -h /home

# 查看 inode
df -i

# POSIX 格式（無單位）
df -P
```

## 檔案系統類型

`df` 輸出通常包含檔案系統類型：

```bash
df -T
Filesystem     Type  1M-blocks  Used Available Use% Mounted on
/dev/sda1      ext4     51200  25600     24000  52% /
tmpfs          tmpfs     2048     64      1984   4% /tmp
```

常見類型：
- `ext4`：Linux 擴展檔案系統
- `tmpfs`：記憶體檔案系統
- `nfs`：網路檔案系統
- `vfat`：FAT 檔案系統

## 臨界值警告

`df` 輸出中，當使用率超過一定程度（通常 90%）會有不同的視覺效果：
- 數值顯示
- 紅色警告（某些版本）
- 驚嘆號

## inode 使用

```bash
df -i
```

inode 耗盡時，即使磁碟還有空間，也無法建立新檔案。

## 跨平台差異

- Linux：`/proc/mounts`
- macOS：`/etc/mtab` 或直接查詢
- BSD：`/etc/mtab` 或 `getmntinfo`

## 底層系統呼叫

- `statfs`/`statvfs`：讀取檔案系統統計
- `getmntent`：解析 `/etc/mtab`

## 安全考量

`df` 通常需要讀取 `/proc` 或 `/etc/mtab`，不需要特殊權限。

## 實用範例

```bash
# 檢查根分割區
df -h /

# 檢查所有，排序
df -h | sort -k5 -h

# 檢查遠端掛載
df -h nfs.example.com:/share
```

## 相關指令

- `du`：目錄佔用
- `ncdu`：互動式磁碟分析
- `duf`：現代化替代