# od — 八進位傾印

`od`（octal dump）以八進位格式顯示檔案內容。

## 核心設計

```rust
let bytes_per_group = 2;
let groups_per_line = 8;

for (chunk_idx, chunk) in data.chunks(bytes_per_group * groups_per_line).enumerate() {
    let addr = chunk_idx * bytes_per_group * groups_per_line;
    print!("{:07o}", addr);

    for group in chunk.chunks(bytes_per_group) {
        let val = if group.len() >= 2 {
            (group[0] as u16) | ((group[1] as u16) << 8)
        } else {
            group[0] as u16
        };
        print!(" {:06o}", val);
    }
    println!();
}
```

`od` 以 2 位元組（16 位元）為組，8 組為一行輸出。

## 輸出格式

```
0000000 056142 066154 061565 066545 064556 062555 005012 005012
0000020 066554 062465 066155 064555 066142 005012 005012 005012
```

每行：
- 第 1-7 欄：位址（八進位）
- 其後每 6 欄：兩位元組的資料（八進位）

## 位址計算

```rust
let addr = chunk_idx * bytes_per_group * groups_per_line;
print!("{:07o}", addr);
```

位址從 0 開始（八進位），每行遞增 16（0o20）。

## 為何用八進位？

- 早期電腦以 12 位元（3 位元組）或 18 位元為單位
- 八進位每數位代表 3 個位元，方便轉換
- 現代工具也有 `-d`（十進位）、`-x`（十六進位）選項

## od 的用途

### 檢視二進制檔案
```bash
od /bin/ls | head
```

### 檢查字元編碼
```bash
echo -e "hello\n" | od -c
```

### 提取二進制資料
```bash
od -An -td4 -N 16  # 以有號十進位讀取前 16 位元組
```

## 字元顯示模式

完整 `od` 支援：
- `-c`：ASCII 字元
- `-s`：短整數
- `-i`：整數
- `-l`：長整數
- `-o`：八進位（預設）

## 與 hexdump 的比較

| 特性 | `od` | `hexdump`/`xxd` |
|------|------|----------------|
| 預設格式 | 八進位 | 十六進位 |
| 起源 | POSIX | BSD/Linux |
| 功能 | 簡單 | 更豐富 |

## 典型用途

### 檢查不可見字元
```bash
cat -A file.txt | od -c
```

### 教學用途
```bash
od -c /bin/ls  # 顯示可見字元
od -o /bin/ls  # 顯示八進位
```

### 檔案完整性
```bash
od -A x -t x1z file  # 十六進位 + ASCII
```

## 位址偏移

```bash
od -A d  # 十進位位址
od -A x  # 十六進位位址
od -A n  # 不顯示位址
```

## 輸出範例

原始資料（ASCII "Hello\n"）：
```
0000000 110 145 154 154 111 012
```

每位元組顯示為八進位。

## 安全性

`od` 只讀取不寫入，是安全的操作。

## 底層系統呼叫

`od` 使用：
- `read()`：讀取檔案
- 標準輸出

## 實用範例

```bash
# 基本傾印
od file.txt

# 只看開頭
od -N 32 file.bin

# 跳過開頭
od -j 256 file.bin

# 十進位格式
od -t d1 file.txt

# 合併十六進位和 ASCII
od -A x -t x1z file
```

## 與其他工具的比較

- `od`：八進位，標準
- `hexdump -C`：十六進位 + ASCII
- `xxd`：十六進位 + ASCII + 反斜線轉義
- `strings`：提取可讀字串

## 相關指令

- `hexdump`：十六進位傾印
- `xxd`：十六進位編輯器
- `strings`：提取字串