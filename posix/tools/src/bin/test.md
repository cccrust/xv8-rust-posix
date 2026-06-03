# test — 條件表達式評估

`test`（也写作 `[`）是 POSIX shell 中最常用的條件判斷命令。

## 核心設計

`test` 評估表達式並根據結果返回：
- **true（表達式為真）**：exit code 0
- **false（表達式為假）**：exit code 1
- **error（語法錯誤）**：exit code 2

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    // POSIX test evaluates expressions and returns 0 (true) or 1 (false)
    let mut arg_end = args.len();
    if args.len() > 1 && args[args.len() - 1] == "]" {
        arg_end = args.len() - 1;
    }
    let test_args: Vec<&str> = args[arg_start..arg_end].iter().map(String::as_str).collect();
}
```

## 作為 `[` 命令

`[` 是 `test` 的另一種形式，行為相同但要求最後一個參數是 `]`：

```bash
if [ -f file.txt ]; then  # 等價於 test -f file.txt
    echo "file exists"
fi
```

注意：`[` 和 `]` 之間需要空格。

## 字串測試

```rust
// -n STRING：字串非空
if test_args.len() == 2 && test_args[0] == "-n" {
    exit(if test_args[1].is_empty() { 1 } else { 0 });
}

// -z STRING：字串為空
if test_args.len() == 2 && test_args[0] == "-z" {
    exit(if test_args[1].is_empty() { 0 } else { 1 });
}

// STRING = STRING：相等
// STRING != STRING：不相等
```

## 數值比較

```rust
// -eq：等於 (equal)
test 5 -eq 5   # true

// -ne：不等於 (not equal)
test 5 -ne 3   # true

// -lt：小於 (less than)
test 3 -lt 5   # true

// -le：小於等於 (less than or equal)
test 3 -le 5   # true

// -gt：大於 (greater than)
test 5 -gt 3   # true

// -ge：大於等於 (greater than or equal)
test 5 -ge 5   # true
```

注意：這些是**數值**比較，不是字串比較。"3" 小於 "10" 在數值上是 true，但在字串比較時 "10" 小於 "3"（因為 "1" < "3"）。

## 檔案測試

```rust
// -e PATH：檔案存在
// -f PATH：普通檔案
// -d PATH：目錄
// -r PATH：可讀
// -w PATH：可寫
```

## 邏輯運算

```rust
// ! EXPR：邏輯非
if [ ! -f file.txt ]; then
    echo "file does not exist"
fi

// -a：邏輯與（已廢棄）
// -o：邏輯或（已廢棄）

// 組合使用
if [ -f file.txt -a -r file.txt ]; then
    cat file.txt
fi
```

## 退出碼的反向意義

`test` 的退出碼與常見的 true/false 邏輯**相反**：
- exit 0 = true（條件成立）
- exit 1 = false（條件不成立）

這是因為 `test` 是命令，0 表示「成功執行」。

## Shell 中的使用

```bash
# 基本 if 語句
if [ "$var" = "value" ]; then
    echo "matched"
fi

# 單獨使用
test -f file.txt && echo "exists"

# 取反
if [ ! -d dir ]; then
    mkdir dir
fi
```

## 空白的重要性

```bash
[ "$var" = "value" ]  # 正確：[] 和內容之間有空格
["$var" = "value"]    # 錯誤

# 當變數為空時
[ $var = "value" ]    # 錯誤：空字串會導致語法錯誤
[ "$var" = "value" ]  # 正確
```

## 常見陷阱

1. **使用 `-eq` 比較字串**
   ```bash
   [ "3" -eq "10" ]  # 結果為 true（數值比較）
   [ "3" = "10" ]     # 結果為 false（字串比較）
   ```

2. **未引用的變數**
   ```bash
   var=""
   [ $var = "" ]     # 錯誤
   [ "$var" = "" ]    # 正確
   ```

3. **額外的參數**
   ```bash
   [ -n ]            # true（-n 被當作字串）
   [ -n "" ]         # false（空字串）
   ```

## 與其他語言的比較

- C：`test` 的退出碼 0 = true，C 的 0 = false
- Python：`test` 的退出碼 0 = True，但 exit(0) = True
- Rust：`test` 的退出碼 0 = `Ok(())`，可用於 `std::process::exit(0)`

## 相關指令

- `[`：test 的另一種形式
- `[[`：Bash 擴展的測試（支援更多特性）
- `((`：Bash 的算術測試