# yes — 重複輸出字串

`yes` 持續輸出字串直到被終止，通常用於自動回應提示。

## 核心設計

```rust
fn main() {
    let s = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "y".to_string()
    };

    loop {
        println!("{}", s);
    }
}
```

`yes` 的實現極其簡單：無窮迴圈，不斷輸出字串。

## 預設行為

不帶參數時，預設輸出 `y`：
```bash
yes
# 輸出：
# y
# y
# y
# ...
```

## 典型用途

### 自動輸入 'y'

```bash
yes | rm -ri /tmp/testdir
# 自動回應所有刪除提示為 'y'
```

### 自動輸入密碼

```bash
yes "password" | su -c "command" user
# 為 su 提供密碼輸入
```

### 自動化腳本

```bash
yes | ./configure
# 對所有 configure 提示回答 'y'
```

## 與管道的結合

`yes` 配合 `head` 可以限制輸出量：

```bash
yes | head -n 10  # 只輸出 10 行
```

## 輸出控制

```bash
yes "done" | head -n 5
# 輸出：
# done
# done
# done
# done
# done
```

## 常見用途

### Debian/Ubuntu 自動回應
```bash
yes | apt-get upgrade
```

### 大檔案產生
```bash
yes "data" | head -c 1M > file.txt
```

### 測試 I/O 效能
```bash
yes | pv > /dev/null
```

## 檔案填補

```bash
yes > file.txt  # 會一直寫入直到磁碟滿或被 Kill
```

## 停止 yes

```bash
# 按 Ctrl+C 終止
# 或者使用 timeout
timeout 5 yes
```

## stdin 的替代

在互動式程式中取代手動輸入：

```bash
yes | interactive_program
```

## 與 xargs 的比較

`yes` 適合無條件輸出，`xargs` 適合從 stdin 讀取代碼。

## 效率

`yes` 是非常簡單的程式，輸出速度極快，可能成為 I/O 瓶頸。

## 格式化輸出

```bash
yes "----" | head -n 10
# 輸出分隔線
```

## 錯誤處理

`yes` 幾乎不會失敗，但寫入失敗（如磁碟滿）時會終止。

## 實用範例

```bash
# 快速建立測試檔案
yes "test data" | head -n 100 > test.txt

# 自動確認多個檔案刪除
yes | rm -i file1 file2 file3

# 分隔線
yes = | head -n 3
# =
# =
# =
```

## 與其他工具的組合

```bash
# 測試磁碟寫入
yes | dd of=/dev/null

# 模擬使用者輸入
yes "y" | patch -p1 < fix.patch

# 產生重複資料
yes "repeat" | tr '\n' ' ' | head -c 100
```

## 底層系統呼叫

`yes` 只使用：
- `write()`：持續寫入 stdout

## 安全考量

使用 `yes | rm` 可能導致意外刪除資料，應谨慎使用。

## 相關指令

- `true`：返回成功（什麼都不輸出）
- `seq`：產生數字序列
- `jot`：BSD 數字產生工具
- `pv`：管道檢視器（monitor）