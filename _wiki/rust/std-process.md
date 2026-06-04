# std::process

程序管理和執行。

## Command

執行外部命令：

```rust
use std::process::Command;

let output = Command::new("ls")
    .arg("-la")
    .spawn()
    .expect("Failed to spawn");
```

### 常見用法

```rust
use std::process::Command;

// 簡單執行
let status = Command::new("echo")
    .arg("Hello")
    .status()?;

// 捕獲輸出
let output = Command::new("ls")
    .arg("/tmp")
    .output()?;
println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
```

## Stdio

配置輸入輸出：

```rust
use std::process::{Command, Stdio};
use std::io;

let child = Command::new("ls")
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .spawn()?;
```

### 重定向

```rust
use std::process::{Command, Stdio};
use std::fs::File;

let file = File::create("output.txt")?;
Command::new("ls")
    .stdout(file)
    .spawn()?;
```

## exit

立即退出程式：

```rust
use std::process;

process::exit(0);   // 成功
process::exit(1);   // 失敗
process::exit(2);   // 其他錯誤
```

### ExitCode

```rust
use std::process::ExitCode;

impl ExitCode {
    pub const SUCCESS: ExitCode;
    pub const FAILURE: ExitCode;
}
```

## wait /id

等待程序結束：

```rust
use std::process::Command;

let mut child = Command::new("sleep").arg("1").spawn()?;
let status = child.wait()?;
println!("Exited with: {}", status);
```

## kill

```rust
use std::process::Command;

let mut child = Command::new("sleep").arg("10").spawn()?;
child.kill()?;
let status = child.wait()?;
```

## Stdin/Stdout/Stderr

```rust
use std::io::{self, Write};

io::stdout().write_all(b"Hello\n")?;
io::stderr().write_all(b"Error\n")?;
```

## 本專案使用

### env 工具

```rust
// env.rs
use std::process::Command;

if !args.is_empty() {
    Command::new(&args[0])
        .args(&args[1..])
        .exec();
}
```

### which/nice 使用

```rust
// 在 shell 中 fork + exec
let mut child = Command::new(&cmd[0])
    .args(&cmd[1..])
    .envs(&variables)
    .spawn()?;
```

### kill 工具

```rust
use std::process::Command;

Command::new("kill")
    .arg(format!("-{}", signum))
    .arg(pid.to_string())
    .status()?;
```

## CommandExt (Unix)

```rust
use std::os::unix::process::CommandExt;

Command::new("ls")
    .uid(1000)   // 設定 UID
    .gid(1000)   // 設定 GID
    .spawn()?;
```

## 管道

```rust
use std::process::{Command, Stdio};
use std::io;

let ls = Command::new("ls")
    .stdout(Stdio::piped())
    .spawn()?;

let wc = Command::new("wc")
    .stdin(ls.stdout.unwrap())
    .output()?;
```

## 環境變數繼承

```rust
Command::new("cmd")
    .env_clear()           // 清除所有環境變數
    .env("PATH", "/bin")   // 僅設定 PATH
    .spawn()?;
```

## 輸出捕獲

```rust
let output = Command::new("echo")
    .arg("test")
    .output()?;

output.status    // ExitStatus
output.stdout   // Vec<u8>
output.stderr   // Vec<u8>
```

## 錯誤處理

```rust
use std::process::Command;

match Command::new("nonexistent").spawn() {
    Ok(child) => { /* 成功 */ }
    Err(e) => eprintln!("Failed: {}", e),
}
```

## 底層機制

- **Linux/macOS**：fork + execve
- **xv8**：fork + exec syscall

## POSIX 對應

| Rust | POSIX |
|------|-------|
| `Command::new` | fork + exec |
| `child.wait` | waitpid |
| `process::exit` | _exit |

## 相關模組

- `std::env`：環境變數
- `std::os::unix::process`：Unix 特定功能