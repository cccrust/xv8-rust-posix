# std::env

環境變數和命令行參數處理。

## 命令行參數

```rust
use std::env;

let args: Vec<String> = env::args().collect();
```

`args[0]` 是程式名稱，`args[1..]` 是實際參數。

xv8 工具的使用方式：
```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: command <arg>");
        std::process::exit(1);
    }
    // ...
}
```

## 環境變數

### getenv / setenv / remove_env

```rust
use std::env;

let path = env::var("PATH").unwrap_or_default();
env::set_var("MY_VAR", "value");
env::remove_var("OLD_VAR");
```

### var / var_os

```rust
let home = env::var("HOME").expect("HOME not set");
// var_os 返回 Option<OsString>
let lang = env::var_os("LANG");
```

## current_dir / set_current_dir

```rust
use std::env;

let cwd = env::current_dir()?;
env::set_current_dir("/new/dir")?;
```

## 常用環境變數

| 變數 | 用途 |
|------|------|
| `PATH` | 搜尋路徑 |
| `HOME` | 家目錄 |
| `USER` | 用戶名 |
| `SHELL` | 登入 shell |
| `PWD` | 當前目錄 |

xv8 中的 shell 使用：
```rust
let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
let path = env::var("PATH").unwrap_or_else(|_| "/bin:/usr/bin".to_string());
```

## args_os

```rust
let args: Vec<OsString> = env::args_os().collect();
```

返回 `OsString` 而非 `String`，處理非 UTF-8 參數。

## current_executable

```rust
use std::env;

let exe = env::current_executable()?;
```

取得目前執行檔路徑。

## 錯誤處理

```rust
use std::env;

match env::var("MISSING") {
    Ok(val) => println!("{}", val),
    Err(e) => println!("Not found: {}", e),
}
```

## 底層機制

- **Linux/macOS**：調用 libc getenv/setenv
- **xv8**：通過 getenv/setenv syscall

## 安全性考量

- 避免信任環境變數攻擊
- PATH 搜索可能指向惡意程式
- 使用 `env::current_executable` 而非 argv[0]

## 與 std::process 的關係

```rust
use std::env;
use std::process;

env::args();     // 命令行參數
process::exit(); // 退出程式
```

## xv8 工具中的模式

```rust
let args: Vec<String> = std::env::args().collect();
let mut i = 1;
while i < args.len() && args[i].starts_with('-') {
    // 解析選項
    i += 1;
}
let file = if i < args.len() { &args[i] } else { "-" };
```

## POSIX 對應

| Rust | POSIX |
|------|-------|
| `env::args` | `getopt`/`argc, argv` |
| `env::var` | `getenv` |
| `env::set_var` | `setenv` |

## 相關模組

- `std::process`：程序控制
- `std::path`：路徑處理