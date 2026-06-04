# tee — 複製輸入到輸出和檔案

`tee` 的名字來自水管工人的「T 形接頭」，它將輸入流一分為二：一路到標準輸出，一路到檔案。

## 核心設計

`tee` 的關鍵是同時輸出到多個目的地：

```rust
loop {
    let n = stdio_read(&stdin, &mut buf)?;
    if n == 0 { break; }

    // 輸出到 stdout
    if io::stdout().write_all(&buf[..n]).is_err() { break; }

    // 輸出到每個檔案
    for w in &mut writers {
        if w.write_all(&buf[..n]).is_err() { break; }
    }
}
```

## 資料流

```
stdin ──┬──> stdout
        │
        └──> file1
        │
        └──> file2
        │
        └──> ...
```

`tee` 不改變資料，只是複製它。

## 選項處理

```rust
let mut append = false;  // -a：附加到檔案而非覆寫
```

- 無 `-a`：覆寫目標檔案
- `-a`：附加到目標檔案

## 多檔案支援

```rust
for fname in &files {
    let file = if append {
        File::options().append(true).create(true).open(Path::new(fname))
    } else {
        File::create(Path::new(fname))
    };
    writers.push(Box::new(f));
}
```

`tee` 支援同時寫入多個檔案。

## 緩衝區設計

```rust
let mut buf = [0u8; 8192];
loop {
    let n = stdio_read(&stdin, &mut buf)?;
    // ...
}
```

使用 8KB 緩衝區，在效率和記憶體之間取得平衡。

## 寫入錯誤處理

```rust
for w in &mut writers {
    if w.write_all(&buf[..n]).is_err() { break; }
}
```

寫入錯誤時提前結束，但不會中斷整體處理。

## 典型用途

### 基本用法
```bash
# 同時查看和保存輸出
ls -la | tee output.txt

# 追加模式
echo "new line" | tee -a output.txt
```

### 組合使用

```bash
# tee 並傳遞給其他命令
echo "content" | tee file.txt | wc -l

# 記錄並執行
echo "rm -rf /tmp/*" | tee commands.sh | bash

# 日誌記錄
make 2>&1 | tee build.log
```

### 應用場景

1. **對話式安裝的紀錄**
   ```bash
   ./install.sh 2>&1 | tee install.log
   ```

2. **除錯時保留輸出**
   ```bash
   command 2>&1 | tee output.txt
   ```

3. **同時查看進度和保存**
   ```bash
   long_running_command | tee result.txt
   ```

## 與管道結合

`tee` 可以放在管道的任何位置：

```bash
# 讀取、tee、保存、再傳遞
cat input.txt | tee intermediate.txt | sort > sorted.txt

# 複雜的資料流
head -n 100 data.csv | tee head.csv | grep "pattern" | wc -l
```

## 與重定向的比較

```bash
# tee（可見的輸出 + 儲存）
command | tee file.txt

# 只有重定向（無可見輸出）
command > file.txt
```

`tee` 的優勢是輸出同時可見。

## 效能考量

`tee` 的開銷：
1. 額外的記憶體複製
2. 額外的寫入操作
3. 多個檔案時的磁碟 I/O

對於大檔案，可以考慮使用緩衝輸出。

## 符號連結處理

`tee` 會追隨符號連結，寫入連結指向的檔案（如果允許）。

## 權限和 creat 模式

```rust
File::create()  // 使用 0o666 許可權（受 umask 影響）
```

目標檔案的許可權取決於系統的 umask 設定。

## 底層系統呼叫

- `read(fd, buf, n)`：從 stdin 讀取
- `write(fd, buf, n)`：寫入 stdout 和檔案
- `open(path, flags, mode)`：開啟或建立檔案

## 實用範例

```bash
# 同時儲存乾淨和骯髒的輸出
command 2>&1 | tee output.txt

# 多檔案鏡像
echo "data" | tee file1.txt file2.txt file3.txt

# tee 後接 gzip
echo "content" | tee | gzip > output.gz

# 記錄安裝過程
./install.sh 2>&1 | tee install.log
```

## 變體

- `pee`：類似 tee 但使用管道而非檔案
- `sponge`：從 stdin 讀取，寫入檔案（等 tee 的非緩衝版本）

## 與其他工具的結合

```bash
# 結合 grep（只 tee 匹配的行）
ls | tee files.txt | grep "\.txt"

# 結合 sort
cat file.txt | tee sorted_copy.txt | sort

# 即時監控（結合 less）
command | tee output.txt | less
```

## 相關指令

- `sponge`：吸收輸入再寫入檔案
- `dd`：資料複製（可用於 tee 的類似功能）