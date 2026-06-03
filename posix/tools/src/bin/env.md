# env — 顯示或執行環境

`env` 用於顯示環境變數，或在修改後的環境中執行命令。

## 核心設計

```rust
fn main() {
    let mut command = std::process::Command::new(&cmd[0]);
    command.args(&cmd[1..]);

    if ignore_env {
        command.env_clear();  // 清除所有環境
    }

    for (k, v) in &set_vars {
        command.env(k, v);    // 設定環境變數
    }

    for k in &unset_vars {
        command.env_remove(k);  // 移除環境變數
    }

    command.status()?;
}
```

## 選項處理

```rust
match args[i].as_str() {
    "-i" => { ignore_env = true; }  // 清除現有環境
    "-u" => { unset_vars.push(args[i + 1]); }  // 移除特定變數
    "--" => { break; }
}
```

## 四種主要用法

### 1. 顯示所有環境變數

```bash
env
# 輸出：
# HOME=/home/user
# PATH=/usr/local/bin:/usr/bin
# ...
```

### 2. 在乾淨環境中執行

```bash
env -i /bin/sh  # 清除所有環境，啟動乾淨的 shell
```

### 3. 設定變數並執行

```bash
env VAR=value command
# 等價於
VAR=value command
```

### 4. 移除變數並執行

```bash
env -u HOME /bin/sh  # 移除 HOME，啟動 shell
```

## 環境清除（-i）

```rust
if ignore_env {
    command.env_clear();
}
```

`env_clear()` 清除命令的所有環境變數，只保留明確指定的。

## 設定和移除

```rust
for (k, v) in &set_vars {
    command.env(k, v);
}
for k in &unset_vars {
    command.env_remove(k);
}
```

## 環境變數繼承

不帶任何選項時，`env` 會把當前環境傳遞給子程序，這是隱式行為。

## 典型用途

### Shebang 的應用
```bash
#!/usr/bin/env python3
```

這行告訴系統使用 `env` 找到 `python3`，在 PATH 中搜尋。

### PATH 擴展
```bash
env PATH="/usr/local/bin:$PATH" command
```

### 乾淨環境測試
```bash
env -i HOME=/home/user PATH=/usr/bin command
```

### 環境注入
```bash
env DEBUG=1 ./program
```

## 與 export 的比較

```bash
# env -i 清除並重新設定
env -i HOME=/tmp bash

# export 在當前 shell 中設定
export HOME=/tmp
```

`env` 主要用於**子程序**，而 `export` 用於**當前 shell**。

## 安全考量

使用 `env` 執行未信任的命令時要注意：
- 不要在 PATH 中留下空路徑（安全漏洞）
- `env -i` 清除所有環境，包括 PATH

## 底層系統呼叫

`env` 的核心是建立帶有修改過的環境的子程序：

```rust
command.env(k, v)    // 對應 fork + execve + 環境陣列
```

## 與 setsid 的比較

`setsid` 建立新的 session，而 `env` 專注於環境變數管理。

## 實用範例

```bash
# 使用乾淨環境執行腳本
env -i PATH=/usr/local/bin:/usr/bin /path/to/script.sh

# Python 多版本管理
/usr/bin/env python2
/usr/bin/env python3

# 臨時覆蓋變數
env LANG=en_US.UTF-8 ./program

# Debug 模式
env DEBUG=1 VERBOSE=1 ./program
```

## POSIX 規範

POSIX 定義了 `env` 的基本行為：
- 沒有選項：顯示環境並 exit（0）
- `-i`：忽略導入的環境
- `name=value`：設定變數
- `command`：執行命令

## 相關指令

- `export`：在 shell 中導出變數
- `set`：顯示/設定 shell 選項
- `unset`：刪除 shell 變數
- `printenv`：顯示環境變數（功能類似）