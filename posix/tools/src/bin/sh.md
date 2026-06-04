# sh — POSIX 標準 Shell

`sh`（Bourne Shell）是 POSIX 標準的 shell 實現，提供了命令列解釋器的完整功能。

## Shell 的核心組件

`sh` 的實現包含幾個關鍵部分：

### ShellContext — 執行期狀態

```rust
struct ShellContext {
    vars: HashMap<String, String>,       // Shell 變數
    exported: Vec<String>,               // 導出的環境變數
    readonly: Vec<String>,               // 唯讀變數
    last_status: i32,                    // 上一個命令的退出碼
    last_bg_pid: Option<usize>,          // 最後後台程序的 PID
    shell_pid: u32,                       // Shell 本身的 PID
    positional: Vec<String>,             // 位置參數 $1, $2, ...
    funcs: HashMap<String, Vec<Vec<String>>>,  // Shell 函數
    loop_level: usize,                   // 巢狀迴圈層級
    traps: HashMap<String, String>,       // 訊號處理
}
```

## 變數擴展

Shell 最基本的功能之一是變數擴展：

```rust
fn get_var(&self, name: &str) -> String {
    if name == "?" { return self.last_status.to_string(); }
    if name == "$" { return self.shell_pid.to_string(); }
    if name == "!" { return self.last_bg_pid.map(|p| p.to_string()).unwrap_or_default(); }
    if name == "#" { return self.positional.len().to_string(); }
    // $1, $2, ... 位置參數
    if let Ok(n) = name.parse::<usize>() {
        if n > 0 && n <= self.positional.len() {
            return self.positional[n - 1].clone();
        }
    }
    self.vars.get(name).cloned().unwrap_or_default()
}
```

## Tokenizer（詞彙分析）

將輸入字串分解為 tokens：

```rust
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut in_dquote = false;
    let mut escape = false;
    // ...
}
```

處理的 token 類型：
- **普通單詞**：命令和參數
- **引號**：`'`（單引號）、`"`（雙引號）
- **管道**：`|`
- **重定向**：`>`、`>>`、`<`、`<<`、`>&`
- **控制結構**：`if`、`then`、`fi`、`for`、`while`、`do`、`done`

## 管道處理

管道連接兩個命令的輸出和輸入：

```rust
let mut parts = line.split('|').collect::<Vec<_>>();
for (i, part) in parts.iter().enumerate() {
    let cmd = parse_command(part);
    // 設定 stdin/stdout 重定向
}
```

## 重定向

Shell 支援豐富的 I/O 重定向：

```rust
struct Redirect {
    fd: u32,           // 檔案描述符（0=stdin, 1=stdout, 2=stderr）
    op: String,        // 重定向操作
    target: String,    // 目標檔案或描述符
    heredoc_content: Option<String>,
}
```

支援的重定向：
- `cmd > file`：stdout 重新導向到檔案（覆寫）
- `cmd >> file`：stdout 追加到檔案
- `cmd < file`：從檔案讀取 stdin
- `cmd 2>&1`：stderr 重新導向到 stdout
- `cmd << EOF`：here document

## 命令執行

```rust
let mut command = Command::new(&cmd_name);
command.args(&args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit());

let status = command.status()?;
```

Shell 不直接執行命令，而是 fork 一個子程序並 exec。

## 內建命令

某些命令是 shell 內建，不需要外部程式：

- `cd`：變更目錄
- `export`：導出環境變數
- `readonly`：設定唯讀變數
- `set`：設定 shell 選項
- `unset`：刪除變數

```rust
fn run_builtin(cmd: &str, args: &[String]) -> Option<i32> {
    match cmd {
        "cd" => { chdir(&args[1]); Some(0) }
        "export" => { /* ... */ Some(0) }
        _ => None,
    }
}
```

## 控制流

Shell 支援完整的控制結構：

### if 語句
```bash
if [ condition ]; then
    commands
elif [ condition ]; then
    commands
else
    commands
fi
```

### for 迴圈
```bash
for var in list; do
    commands
done
```

### while 迴圈
```bash
while condition; do
    commands
done
```

## 變數擴展

支援豐富的變數擴展語法：

```bash
${var:-default}    # 如果 var 為空，使用 default
${var:=default}    # 如果 var 為空，設為 default 並使用
${var:+alt}        # 如果 var 非空，使用 alt
${var:?error}      # 如果 var 為空，輸出 error 並退出
${#var}            # var 的長度
${var%pattern}     # 從結尾移除最短匹配
${var%%pattern}    # 從結尾移除最長匹配
${var#pattern}     # 從開頭移除最短匹配
${var##pattern}    # 從開頭移除最長匹配
```

## 命令替換

```bash
output=$(command)  # 執行 command，用輸出替換
output=`command`   # 同上，較老語法
```

## 算術擴展

```bash
result=$((a + b))   # 算術表達式
```

## 訊號處理

Shell 可以攔截和處理訊號：

```rust
traps: HashMap<String, String>,  // trap "handler" SIGNAL
```

例如：
- `trap echo Interrupted INT` — 收到 SIGINT 時輸出訊息

## 與 bash 的差異

xv8 的 `sh` 提供了 POSIX 相容功能，但不如 bash 擴展：
- 不支援陣列
- 不支援更復雜的歷史 expansion
- `[[]]` 和 `(())` 不是內建

## REPL 迴圈

Shell 作為互動式解釋器，運行 REPL：

```rust
loop {
    print!("$ ");
    let input = readline();
    let tokens = tokenize(&input);
    for cmd in parse_commands(tokens) {
        execute(cmd);
    }
}
```

## 相關主題

- [[Shell]]：Shell 的一般概念
- [[Syscall]]：fork、exec、wait 系統呼叫
- [[Process]]：子程序的建立和管理