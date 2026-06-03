# echo — 顯示文字

`echo` 是最簡單的 shell 内建命令，用於將文字輸出到標準輸出。

## 核心功能

`echo` 的任務很簡單：將參數（以空格分隔）輸出到 stdout：

```rust
args[i..].join(" ")
```

然後通常會輸出換行符（`println!`）。

## 轉義序列處理

`-e` 選項開啟對轉義序列的解釋：

```rust
fn escape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),   // 換行
                Some('t') => out.push('\t'),   // Tab
                Some('r') => out.push('\r'),   // 回車
                Some('\\') => out.push('\\'), // 反斜線
                Some('0') => out.push('\0'),   // Null
                Some('a') => out.push('\x07'), // 警示聲
                Some('b') => out.push('\x08'), // 退格
                Some('v') => out.push('\x0b'), // 垂直 Tab
                Some('f') => out.push('\x0c'), // 換頁
                Some('c') => break,            // \c 後面的不輸出，抑制換行
                Some(c) => { out.push('\\'); out.push(c); }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
```

## 選項解析的細節

```rust
let mut no_newline = false;       // -n：不輸出換行
let mut interpret_escapes = false; // -e：解釋轉義，-E：不解釋
```

```rust
// GNU echo 行為：遇到非選項字元就停止解析
if args[i].starts_with('-') && args[i].len() > 1 {
    let rest: String = args[i][1..].chars().filter(|&c| c == 'n' || c == 'e' || c == 'E').collect();
    if rest.len() < args[i].len() - 1 { break; }
}
```

這確保 `echo -n abc` 被正確解析。

## \c 抑制換行的實現

`\c` 的特殊處理在於直接 `break`，不放任何東西到輸出，且不輸出換行：

```rust
Some('c') => break,  // 直接結束，不加任何東西
```

## Shell 內建 vs 獨立程式

在大多數 shell 中，`echo` 是內建命令，因為：
1. 效能：不需要建立新程序
2. 簡單：無需獨立程式

但 `posix/tools` 中的 `echo` 是獨立的 binary，可以在腳本中使用。

## 與 printf 的比較

`printf` 是更強大的格式化輸出命令：
- `printf` 需要明確指定格式字串
- `printf` 不自動加換行
- `printf` 支援更多格式化選項

```bash
echo "Hello\n"    # 可能輸出 \n 作為字面量（取決於 shell）
printf "Hello\n"  # 總是輸出換行
```

## 跨平台差異

不同系統的 `echo` 行為不同：
- GNU coreutils：`echo -e "a\nb"` 輸出換行
- BSD：`echo -e` 可能不支援 `-e`
- POSIX shell：echo 的轉義行為是未定義的

因此可移植的腳本應使用 `printf`。

## 底層系統呼叫

`echo` 最終使用：
- `write(fd, buf, n)`：寫入輸出

## 典型用途

```bash
echo "Hello, World!"
echo -n "No newline"
echo -e "Tab\tbetween\twords"
echo $variable
echo $(date)
```

## 環境變數擴展

Shell 中的 `echo $VAR` 會先擴展變數再輸出。xv8 的 `echo` 是獨立程式，不進行 shell 擴展，所以：
```bash
echo $HOME        # 在 shell 中：擴展後輸出
./echo $HOME      # 直接呼叫：輸出 $HOME（不擴展）
```

## 相關指令

- `printf`：格式化輸出
- `puts`：類似 echo 但返回結果
- `print`：某些 shell 的內建