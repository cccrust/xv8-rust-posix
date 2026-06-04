# Signal — 訊號機制

訊號是軟體中斷，用於通知程序發生了事件。

## 概述

```
程序 A                 程序 B
   │                      │
   │  kill(pid, SIGTERM)  │
   └──────────────────────►│
                           │
                    收到 SIGTERM
                           │
                    終止或處理
```

## 常見訊號

| 訊號 | 編號 | 預設動作 | 說明 |
|------|------|----------|------|
| SIGHUP | 1 | 終止 | 掛斷 |
| SIGINT | 2 | 終止 | Ctrl+C |
| SIGQUIT | 3 | 傾印 | Ctrl+\ |
| SIGKILL | 9 | 終止 | 無法忽略 |
| SIGTERM | 15 | 終止 | 優雅終止 |
| SIGSEGV | 11 | 傾印 | 段錯誤 |
| SIGALRM | 14 | 終止 | 計時器 |
| SIGCHLD | 17 | 忽略 | 子程序結束 |

## signal / sigaction

### signal（簡單）

```c
typedef void (*sighandler_t)(int);
sighandler_t signal(int signum, sighandler_t handler);

void handler(int sig) {
    printf("Caught signal %d\n", sig);
}

signal(SIGINT, handler);
```

### sigaction（完整）

```c
struct sigaction {
    void     (*sa_handler)(int);
    void     (*sa_sigaction)(int, siginfo_t *, void *);
    sigset_t sa_mask;
    int      sa_flags;
};

struct sigaction act;
act.sa_handler = handler;
sigemptyset(&act.sa_mask);
act.sa_flags = 0;

sigaction(SIGINT, &act, NULL);
```

## kill — 發送訊號

```c
int kill(pid_t pid, int sig);
// pid > 0: 發送到指定程序
// pid = 0: 發送到同組所有程序
// pid = -1: 發送到所有程序（需要權限）
// pid < -1: 發送到程序組 |-pid|
```

## 預設處理

| 動作 | 說明 |
|------|------|
| Term | 終止程序 |
| Ign | 忽略訊號 |
| Core | 終止並傾印核心 |
| Stop | 暫停程序 |
| Cont | 繼續執行 |

## 程式產生的訊號

### SIGSEGV

記憶體存取錯誤：

```c
int *p = NULL;
*p = 42;  // SIGSEGV
```

### SIGFPE

浮點例外：

```c
int x = 1 / 0;  // SIGFPE
```

### SIGALRM

alarm() 產生：

```c
alarm(5);  // 5 秒後收到 SIGALRM
pause();  // 等待訊號
```

## 子程序與訊號

fork 後子程序繼承訊號處理：
```c
signal(SIGCHLD, handler);  // 子程序也會處理 SIGCHLD
```

exec 後訊號處理保持，但如果處理常式位址無效會導致終止。

## raise — 自我發送

```c
raise(SIGTERM);  // 等同於 kill(getpid(), SIGTERM)
```

## sigprocmask — 阻塞訊號

```c
sigset_t set, oldset;
sigemptyset(&set);
sigaddset(&set, SIGINT);  // 阻塞 SIGINT

sigprocmask(SIG_BLOCK, &set, &oldset);
// 臨界區（SIGINT 被阻塞）
sigprocmask(SIG_SETMASK, &oldset, NULL);  // 恢復
```

## sigpending — 檢查未處理訊號

```c
sigset_t pending;
sigpending(&pending);
if (sigismember(&pending, SIGINT)) {
    // SIGINT 已產生但被阻塞
}
```

## pause — 等待訊號

```c
pause();  // 永遠等待，直到收到訊號
```

## 可重入

訊號處理常式必須是可重入的（不能使用不安全的函式）。

### 安全函式

```c
// 可在訊號處理常式中使用
write()
_Exit()
sigprocmask()
```

### 不安全函式

```c
// 不要在訊號處理常式中使用
printf()   // 可能鎖定
malloc()   // 可能鎖定
```

## sigsetjmp/siglongjmp

跨訊號跳躍：

```c
sigjmp_buf env;

sigsetjmp(env, 1);  // 保存訊號遮罩
// ...
signal(SIGINT, handler);
siglongjmp(env, 1);  // 恢復訊號遮罩並跳躍
```

## 範例：優雅終止

```c
volatile sig_atomic_t running = 1;

void sigint_handler(int sig) {
    running = 0;
}

int main() {
    signal(SIGINT, sigint_handler);
    signal(SIGTERM, sigint_handler);

    while (running) {
        // 處理任務
    }

    // 清理並退出
    return 0;
}
```

## 與 xv6 的差異

| 特性 | xv6 | xv8 |
|------|-----|-----|
| signal() | 無 | 有 |
| sigaction | 無 | 基本 |
| SIGCHLD | 忽略 | 可處理 |
| kill | 基本 | 有 |

## 安全性

- 只有程序的擁有者或 root 可以發送訊號
- SIGKILL 和 SIGSTOP 不能被阻塞或忽略

## 相關主題

- [[Process]]：程序如何處理訊號
- [[Syscall]]：訊號如何觸發 trap