# exec — 程式執行

exec 系列函式用新程式替換目前程序的記憶體。

## exec 家族

| 函式 | 說明 |
|------|------|
| execl | 傳遞可變參數列表 |
| execv | 傳遞參數向量 |
| execle | 傳遞環境 + 列表 |
| execve | 傳遞環境 + 向量 |
| execlp | 使用 PATH + 列表 |
| execvp | 使用 PATH + 向量 |

## execl

```c
int execl(const char *path, const char *arg0, ... /* (char *) NULL */);

execl("/bin/ls", "ls", "-la", NULL);
```

## execv

```c
int execv(const char *path, char *const argv[]);

char *args[] = { "ls", "-la", NULL };
execv("/bin/ls", args);
```

## execlp/execvp

使用 PATH 搜尋：

```c
execlp("ls", "ls", "-la", NULL);  // 搜尋 /bin, /usr/bin 等
execvp("ls", args);
```

## execve（系統呼叫）

```c
int execve(const char *path, char *const argv[], char *const envp[]);

char *args[] = { "ls", NULL };
char *env[] = { "PATH=/bin", "HOME=/root", NULL };
execve("/bin/ls", args, env);
```

注意：只有 execve 是真正的系統呼叫，其他都是包裝。

## 執行後的變化

| 保留 | 說明 |
|------|------|
| PID | 程序 ID 不變 |
| PPID | 父程序 ID 不變 |
| 開啟的檔案描述符 | 除非設定 O_CLOEXEC |
| 目前目錄 | 不變 |
| 環境變數 | 除非指定新環境 |
| 訊號處理 | 可能被重置 |
| 程式計數器 | 變為新程式入口 |

| 改變 | 說明 |
|------|------|
| 程式碼 | 新程式內容 |
| 堆疊 | 重置為新程式格式 |
| 堆積 | 重置 |
| 全域資料 | 新程式內容 |

## O_CLOEXEC

exec 前應設定 close-on-exec：

```c
int fd = open("file", O_RDONLY | O_CLOEXEC);
// exec 後 fd 會自動關閉
```

或手動：

```c
fcntl(fd, F_SETFD, FD_CLOEXEC);
```

## 錯誤處理

```c
if (execve(path, args, env) < 0) {
    perror("exec failed");
    exit(1);
}
// 只有失敗才執行這裡
```

## 與 fork 的典型模式

```c
pid_t pid = fork();

if (pid == 0) {
    // 子程序：執行新程式
    execlp("ls", "ls", "-la", NULL);
    _exit(127);  // exec 失敗
}

// 父程序：等待
int status;
waitpid(pid, &status, 0);
```

## 為何 _exit 而非 exit

exec 失敗時必須使用 `_exit`：

- `exit` 會執行 atexit 處理、flush 緩衝區等
- `_exit` 直接終止，不做任何清理
- 避免重複關閉 fd 等資源

## exec 與環境

### 繼承當前環境

```c
execl("/bin/program", "program", NULL);
// 環境變數保持不變
```

### 指定新環境

```c
char *env[] = { "VAR=value", NULL };
execle("/bin/program", "program", NULL, env);
```

## 覆蓋當前程式

如果只是想執行另一個程式而不創建新程序：

```c
// 簡單的 shell 命令執行
execlp("grep", "grep", "pattern", filename, NULL);
// 成功：grep 接管，grep 結束後整個程式結束
```

## 路徑搜尋

execlp/execvp 搜尋 PATH：

```c
// 相當於：
for (dir in PATH) {
    if (access(dir/cmd, X_OK) == 0) {
        execve(dir/cmd, ...);
    }
}
```

PATH 如果為空，使用當前目錄。

## 與 xv6 的差異

| 特性 | xv6 | xv8 |
|------|-----|-----|
| execve | 有 | 有 |
| execlp | 無 | 有 |
| execvp | 無 | 有 |
| ELF 載入 | 有 | 有 |

## 安全考量

### PATH 攻擊

```c
// 危險：
execlp("ls", "ls", NULL);
// 如果 /tmp 中有惡意的 "ls"
// 攻擊者：ln -s /bin/malicious /tmp/ls
// PATH=/tmp:$PATH 執行
```

防範：使用絕對路徑或淨化 PATH。

### FD 洩漏

```c
// 如果不及時關閉 fd
int fd = open("secret", O_RDONLY);
// ...
execlp("program", "program", NULL);
// program 可能繼承 fd 並讀取
```

防範：使用 O_CLOEXEC。

## 實用範例：mysh 的外命令

```rust
pub fn execute_external(&mut self, cmd: &[String]) -> bool {
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        // 子程序
        let args: Vec<*const c_char> = cmd.iter()
            .map(|s| s.as_ptr() as *const c_char)
            .collect();
        args.push(std::ptr::null());

        libc::execvp(args[0], args.as_ptr() as *mut *mut c_char);
        libc::_exit(1);  // exec 失敗
    }
    // 父程序：等待
    let mut status: i32 = 0;
    libc::waitpid(pid, &mut status, 0);
    libc::WIFEXITED(status)
}
```

## 相關主題

- [[Process]]：fork/exec 模式
- [[File-Descriptor]]：fd 繼承
- [[Environment]]：環境變數傳遞