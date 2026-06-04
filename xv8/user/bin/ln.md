# ln — 建立檔案連結

ln 為檔案建立硬連結或符號連結。

## 使用方式

```bash
ln old new
```

## 實作

```rust
fn main(args: Args) {
    if args.len() < 2 {
        exit_with_msg("usage: ln old new");
    }

    let old = args.get_str(1).expect("old to be str");
    let new = args.get_str(2).expect("new to be str");

    if let Err(e) = link(old, new) {
        eprintln!("ln: {} ({} -> {})", e, old, new);
    }
}
```

## 連結類型

### 硬連結

```rust
link(old, new)  // 使用 link() 系統呼叫
```

- 多個連結指向同一個 inode
- 刪除任一連結不影響其他
- 不能跨檔案系統
- 不能連結目錄

### 符號連結（未實現）

符號連結是特殊檔案，內容是另一個檔案的路徑。

## 行為

- 如果 `old` 不存在，輸出錯誤
- 如果 `new` 已存在，輸出錯誤（不覆蓋）
- 使用 `link()` 系統呼叫建立連結

## 錯誤處理

| 錯誤 | 說明 |
|------|------|
| NoEntry | 來源檔案不存在 |
| AlreadyExists | 目標已存在 |
| CrossDeviceLink | 跨檔案系統 |
| IsDirectory | 嘗試連結目錄 |

## 範例

```bash
ln file.txt link.txt     # 建立硬連結
```

## inode 連結計數

每個 inode 有 `nlink` 計數：
- 建立連結時遞增
- 刪除連結時遞減
- `nlink = 0` 時釋放磁碟空間

## 相關主題

- [[rm]]：刪除檔案
- [[unlink]]：刪除連結