# ls — 列出目錄內容

ls 列出檔案和目錄的資訊，支援遞迴列出。

## 使用方式

```bash
ls [-l] [file or directory...]
```

## 輸出格式

```
f 1234   123 file.txt
d    1     0 dir/
- 5678 4096 data.bin
```

格式：`類型  inode  大小  名稱`

| 類型字元 | 說明 |
|----------|------|
| `f` | 一般檔案 |
| `d` | 目錄 |
| `D` | 裝置 |
| `l` | 符號連結 |
| `p` | FIFO |

## 實作

```rust
fn ls(path: &str) {
    let mut fd = open(path, OpenFlag::READ_ONLY)?;
    let mut stat = Stat::default();
    fstat(fd, &mut stat)?;

    match stat.r#type {
        InodeType::Directory => {
            // 讀取目錄項
            let mut buf = [0u8; size_of::<Directory>()];
            while fd.read(&mut buf) == Ok(buf.len()) {
                let dir = unsafe { &*(buf.as_ptr() as *const Directory) };
                if dir.inum == 0 { continue; }

                // 遞迴處理每個項目
                let full_path = format!("{}/{}", path, dir.name);
                ls(&full_path);
            }
        }
        InodeType::File | InodeType::Device | InodeType::SymLink | InodeType::Fifo => {
            // 輸出檔案資訊
            println!("{} {:>4} {:>8} {}", type_char(stat.r#type), stat.ino, stat.size, path);
        }
    }
}
```

## 目錄讀取

目錄的結構（在磁碟上）：

```rust
struct Directory {
    inum: u16,        // inode 編號
    name: [u8; 14],  // 檔案名稱 (14 bytes)
}
```

ls 讀取目錄的每個目錄項，解析出名稱，然後遞迴呼叫 `ls` 處理子目錄。

## 遞迴行為

- 如果輸入是目錄，ls 會遞迴進入並列出所有內容
- 輸出時顯示完整路徑
- 特殊項目 `.` 和 `..` 被跳過

## 範例

```bash
ls              # 列出目前目錄
ls /            # 列出根目錄
ls /bin /usr    # 列出多個目錄
```

## 與 xv8 核心的整合

- 使用 `fstat` 取得 inode 資訊
- 使用 `open` + `read` 列目錄
- 使用 `close` 關閉檔案描述符

## 相關主題

- [[cat]]：檔案輸出
- [[Stat]]：檔案狀態