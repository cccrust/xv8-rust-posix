# Shell（Shell 殼層）

xv8 提供了一個功能完整的 POSIX shell（sh），支援管道、IO 重新導向、後台任務和命令列編輯。

## Shell 的結構

Shell 由幾個主要部分組成：

1. **詞彙分析器（Lexer）**：將輸入分解為 tokens
2. **解析器（Parser）**：將 tokens 轉換為語法樹
3. **執行引擎**：遍歷語法樹並執行命令
4. **作業控制**：管理前台/後台任務

## 詞彙分析

輸入行被分解為各種類型的 tokens：

```rust
pub enum TokenKind {
    Word,      // 普通單詞（命令、參數）
    PIPE,      // |
    REDIR_IN,  // <
    REDIR_OUT, // >
    REDIR_APP, // >>
    BG,        // &
    SEMICOLON, // ;
    AND,       // &&
    OR,        // ||
}
```

`lexer.rs` 負責：
- 識別普通單詞（不帶引號）
- 處理單引號（'...'）和雙引號（"..."）
- 識別元字元（|、<、>、&、;）
- 處理轉義（\）

## 語法樹

解析器輸出命令的語法樹：

```rust
pub enum Cmd {
    Single(SingleCmd),        // 簡單命令
    Pipe(Box<Cmd>, Box<Cmd>),  // pipe1 | pipe2
    Seq(Box<Cmd>, Box<Cmd>),   // cmd1 ; cmd2
    And(Box<Cmd>, Box<Cmd>),   // cmd1 && cmd2
    Or(Box<Cmd>, Box<Cmd>),    // cmd1 || cmd2
    Subshell(Box<Cmd>),        // ( cmd )
    Redir(Box<Cmd>, Redir),    // cmd < file > file2
    BG(Box<Cmd>),              // cmd &
}
```

## 簡單命令執行

`sh.rs` 中的 `runcmd()` 處理簡單命令：

```rust
fn runcmd(cmd: &mut SingleCmd) -> i32 {
    // 1. 查找命令
    let path = find_cmd(&cmd.name)?;
    // 2. 建立子程序
    let pid = fork();
    if pid == 0 {
        // 子程序：設定 IO、執行命令
        setup_redirects(cmd)?;
        exec(path, &cmd.args)?;
    } else {
        // 父程序：等待完成
        waitpid(pid)
    }
}
```

## 管道（Pipe）

管道連接兩個命令的輸出和輸入：

```
cmd1 | cmd2
```

實現方式：
1. `pipe()` 系統呼叫建立管道，取得兩個 fd（讀端、寫端）
2. fork 兩個子程序
3. cmd1 的 stdout 重定向到管道的寫端
4. cmd2 的 stdin 重定向到管道的讀端
5. 關閉管道的不需要端
6. 兩個子程序並行執行

## IO 重新導向

Shell 支援標準 IO 的重新導向：

- `cmd < file`：將 stdin 重定向為檔案
- `cmd > file`：將 stdout 重定向為檔案（覆寫）
- `cmd >> file`：將 stdout 重定向為檔案（附加）
- `cmd 2> file`：將 stderr 重定向為檔案

```rust
fn setup_redirects(cmd: &mut SingleCmd) -> Result<()> {
    for redir in &cmd.redirs {
        match redir.kind {
            REDIR_IN => {
                let fd = open(redir.path, O_RDONLY)?;
                dup2(fd, STDIN_FILENO)?;
            }
            REDIR_OUT => {
                let fd = open(redir.path, O_WRONLY | O_CREATE | O_TRUNC)?;
                dup2(fd, STDOUT_FILENO)?;
            }
            // ...
        }
    }
    Ok(())
}
```

## 後台執行

`cmd &` 將命令放到後台執行：

1. fork 一個子程序
2. 父程序不等待，立即返回工作階段 ID
3. 子程序成為新的程序群組 leader

## 內建命令

Shell 還實現了一些內建命令，這些命令不在外部程式中，而是在 shell 內部處理：

- `cd`：變更目前目錄
- `pwd`：顯示目前目錄
- `export`：設定環境變數
- `alias`：命令別名
- `set`：顯示/設定 shell 選項

```rust
fn run_builtin(cmd: &mut SingleCmd) -> Option<i32> {
    match cmd.name.as_str() {
        "cd" => {
            chdir(&cmd.args[1])?;
            Some(0)
        }
        "export" => {
            for arg in &cmd.args[1..] {
                setenv_from_eq(arg);
            }
            Some(0)
        }
        _ => None,
    }
}
```

## 命令查找

當執行一個命令時，shell 按以下順序查找：

1. 絕對或相對路徑（如 `/bin/ls` 或 `./a.out`）
2. 環境變數 PATH 中的目錄

```rust
fn find_cmd(name: &str) -> Option<String> {
    if name.contains('/') {
        return Some(name.to_string());
    }
    let path = getenv("PATH")?;
    for dir in path.split(':') {
        let cmd_path = format!("{}/{}", dir, name);
        if exists(&cmd_path) {
            return Some(cmd_path);
        }
    }
    None
}
```

## 命令列編輯

`user/src/line.rs` 實現了簡單的命令列編輯器：

- **方向鍵**：移動游標
- **退格鍵**：刪除字元
- **Enter**：提交命令
- **Ctrl+C**：中斷目前輸入
- **Ctrl+D**：EOF
- **上/下箭頭**：命令歷史導航

```rust
pub struct LineEditor {
    buf: Vec<u8>,       // 目前行緩衝區
    cursor: usize,      // 游標位置
    history: Vec<String>, // 命令歷史
    history_idx: usize, // 歷史導航索引
}
```

## Bash 相容性

xv8 的 bash（`posix/tools/src/bin/bash.rs`）提供了更多 bash 特性的子集：

- `if`/`then`/`fi` 條件判斷
- `for` 迴圈
- `while` 迴圈
- `case` 陳述式
- 命令替換 `$(cmd)`
- 引數展開 `$VAR`、`${VAR}`

## 作業控制

Shell 支援基本的作業控制：

- `jobs`：列出後台任務
- `fg`：將後台任務帶到前景
- `Ctrl+Z`：暫停目前任務

這需要信號機制（SIGTSTP、SIGCONT）和程序群組支援。

## 相關主題

- [[Syscall]]：pipe、fork、exec、wait 等系統呼叫
- [[Process]]：子程序的建立和管理
- [[File-System]]：IO 重新導向與檔案操作