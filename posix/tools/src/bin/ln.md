# ln — 建立檔案連結

`ln`（link）用於建立檔案的連結，分為硬連結和符號連結兩種。

## 連結的基礎概念

在 Unix 檔案系統中，檔案由两部分组成：
1. **inode**：儲存檔案的中繼資料（大小、时间戳、許可權等）
2. **目錄條目**：將檔案名稱對應到 inode

連結是在目錄中建立新的項目，指向同一個 inode。

## 硬連結

硬連結是多個目錄條目指向同一個 inode：

```rust
fs::hard_link(src, &link)
```

特性：
- 所有連結共享同一個 inode
- 刪除任一連結不影響其他連結
- 不能跨檔案系統
- 不能連結目錄
- inode 的連結計數（nlink）記錄有多少連結

## 符號連結（Symlink）

符號連結是一個特殊的小檔案，內容是被連結路徑的字串：

```rust
std::os::unix::fs::symlink(src, &link)
```

特性：
- 有自己的 inode
- 可以跨檔案系統
- 可以連結目錄
- 讀取時會自動解引用（following）
- 目標不存在時形成「斷裂連結」

## 選項處理

```rust
let mut symbolic = false;  // -s：建立符號連結
let mut force = false;     // -f：強制覆寫
```

- `-s`：建立符號連結而非硬連結
- `-f`：如果目標已存在，直接覆寫

## 目標路徑處理

```rust
let link = if link_is_dir {
    link_name.join(src.file_name().unwrap_or_default())
} else {
    link_name.to_path_buf()
};
```

當目標是目錄時，在該目錄下建立與源檔案同名的連結。

## 底層系統呼叫

- `link(oldpath, newpath)`：建立硬連結
- `symlink(target, linkpath)`：建立符號連結
- `lstat(path, buf)`：獲取連結本身資訊（不追隨）
- `readlink(path, buf)`：讀取符號連結的目標

## 許可權模型

硬連結：繼承目標的許可權（不是複製）
符號連結：總是 lrwxrwxrwx（所有權限），實際權限取決於目標

## 刪除連結

刪除連結使用 `unlink`（普通檔案）或 `rmdir`（空目錄）。

## 實用範例

```bash
# 建立符號連結
ln -s /usr/local/bin mybin

# 建立硬連結（備份）
ln file.txt file.txt.bak

# 強制覆寫
ln -sf new_target link_name
```

## 符號連結的應用場景

- **捷徑**：提供易記的名稱訪問深層路徑
- **軟體版本**：-current、-stable 等符號
- **路徑抽象**：程式依賴 `/lib` 而實際是 `/lib64`

## 兩者的比較

| 特性 | 硬連結 | 符號連結 |
|------|--------|----------|
| 跨檔案系統 | 否 | 是 |
| 連結目錄 | 否 | 是 |
| 佔用磁碟空間 | 無（共享 inode） | 小（只存路徑） |
| 斷裂時可見 | 否 | 是 |
| 效能 | 稍好 | 略差（需要解引用） |

## 相關指令

- `ls -l`：顯示連結（`->` 表示符號連結）
- `readlink`：讀取符號連結目標
- `unlink`：移除連結