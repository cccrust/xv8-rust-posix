# sh — Unix 風格 shell

sh 是一個功能完整的命令列解釋器，支援管線、重新導向、後台執行和基本內建命令。

## 支援的功能

| 功能 | 語法 | 說明 |
|------|------|------|
| 執行命令 | `ls -l` | 執行外部程式 |
| 管線 | `ls \| grep foo` | 程序間通訊 |
| 重新導向 | `ls > out.txt` | 輸出到檔案 |
| 附加 | `ls >> out.txt` | 附加到檔案 |
| 輸入重新導向 | `cat < input.txt` | 從檔案讀取 |
| 序列執行 | `ls; echo done` | 依序執行 |
| 後台執行 | `sleep 10 &` | 非同步執行 |
| 環境變數 | `FOO=bar ls` | 設定環境變數 |

## 資料結構

```rust
enum CommandType<'a> {
    Exec { argv_start: usize, argc: usize },      // 基本命令
    Redirect { cmd: usize, file: &'a str, mode: usize, fd: Fd },
    Pipe { left: usize, right: usize },            // 管線
    List { left: usize, right: usize },            // ; 分隔
    Background { cmd: usize },                     // & 後台
}

struct Arena<'a> {
    nodes: [Option<CommandType<'a>>; MAXNODES],  // 命令樹
    argv: [&'a str; MAXARGV],                      // 參數陣列
}
```

## 剖析流程

```
輸入: "ls -l | grep foo > out.txt &"
    │
    ▼
Tokenizer
    │
    ▼
parse_command (遞迴下降剖析)
    │
    ├── parse_exec()     → Exec
    ├── parse_pipe()    → Pipe
    ├── parse_redirect() → Redirect
    ├── parse_list()    → List
    └── parse_background() → Background
    │
    ▼
Command Tree (Arena)
```

## 執行流程

```rust
fn run_command(arena: &mut Arena, cmd: usize) {
    match &arena.nodes[cmd] {
        CommandType::Exec { argv_start, argc } => {
            let argv = &arena.argv[argv_start..argv_start + argc];
            exec(argv[0], argv);
        }
        CommandType::Pipe { left, right } => {
            let (r, w) = pipe();
            // fork, 重定向, 執行左右命令
        }
        CommandType::Redirect { cmd, file, mode, fd } => {
            // 開啟檔案, 重定向 fd
        }
        CommandType::Background { cmd } => {
            // fork, 不等待
        }
    }
}
```

## 環境變數處理

```rust
// 解析: FOO=bar ls
if arg.contains('=') && !arg.starts_with('=') {
    let (k, v) = arg.split_once('=').unwrap();
    setenv(k, v, true);
    // 不執行，只是設定環境
}
```

## 內建命令

| 命令 | 說明 |
|------|------|
| cd [dir] | 改變目錄 |
| exit [n] | 退出 shell |
| pwd | 顯示目前目錄 |
| export | 設定環境變數 |

## 行編輯

sh 支援基本行編輯，包括：
- Backspace 刪除
- Ctrl-C 中斷輸入
- Ctrl-D EOF
- 歷史瀏覽（上行/下行鍵）

## 限制

- 不支援引號嵌套
- 不支援 here documents (`<<`)
- 不支援命令替換 (`` `cmd` ``)
- 不支援算術擴展 (`$((expr))`)

## 相關主題

- [[init]]：啟動 shell
- [[Pipe]]：程序間通訊
- [[exec]]：程式中執行