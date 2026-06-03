# Process — 程序管理

程序是程式的執行個體，擁有獨立的位址空間和系統資源。

## 程序建立：fork

```c
pid_t pid = fork();

if (pid == 0) {
    // 子程序
    exec("/bin/ls", ...);
} else {
    // 父程序
    wait(NULL);  // 等待子程序
}
```

fork 創建一個幾乎完全相同的副本：

### fork 的複製內容

| 項目 | 說明 |
|------|------|
| 程式碼 | 相同（直到 exec）|
| 堆疊 | 複製（Copy-on-Write）|
| 堆積 | 複製（Copy-on-Write）|
| 全域變數 | 複製 |
| fd 表 | 引用相同 file* |
| 目前目錄 | 引用相同 |
| 環境變數 | 複製 |
| 訊號處理 | 複製（但指標相同）|

### Copy-on-Write

fork 後父子共享記憶體，直到其中一方寫入才複製：

```
fork() 後：
  父程序 ─────┬──── 子程序
         (共享，唯讀)
              │
              ▼ 寫入時
         各自複製，獨立修改
```

## exec — 執行新程式

```c
int execv(const char *path, char *const argv[]);
int execve(const char *path, char *const argv[], char *const envp[]);
```

exec 替換目前程序的記憶體：
- 程式碼被新程式替換
- 堆疊被重置
- fd 通常保持開啟（除非設定 O_CLOEXEC）

## wait — 等待子程序

```c
pid_t wait(int *status);
pid_t waitpid(pid_t pid, int *status, int options);
```

### status 檢查

```c
int status;
wait(&status);

if (WIFEXITED(status)) {
    printf("Exited with: %d\n", WEXITSTATUS(status));
} else if (WIFSIGNALED(status)) {
    printf("Killed by signal: %d\n", WTERMSIG(status));
}
```

## exit — 程式結束

```c
void exit(int status);

_Exit(status);  // _Exit 不執行 atexit/flush
```

## 程序結束選項

| 函式 | 沖洗緩衝區 | 執行 atexit |
|------|-----------|-------------|
| exit() | 是 | 是 |
| _Exit() | 否 | 否 |

## getpid/getppid

```c
pid_t getpid(void);   // 目前程序 ID
pid_t getppid(void);  // 父程序 ID
```

## 程序 vs 執行緒

| 特性 | 程序 | 執行緒 |
|------|------|--------|
| 位址空間 | 獨立 | 共享 |
| 資源 | 獨立 | 共享（記憶體、fd）|
| 建立速度 | 慢 | 快 |
| 通訊 | IPC 機制 | 直接記憶體共享 |

## xv8 程序表

```rust
pub const NPROC: usize = 64;  // 最大程序數

pub static mut PROC_TABLE: ProcTable = ProcTable::new();

pub struct ProcTable {
    pub procs: [Option<Process>; NPROC],
}
```

## 程序狀態

```
┌─────────┐
│ UNUSED  │ ← 未使用
└────┬────┘
     │
     ▼
┌─────────┐
│ EMBRYO   │ ← 正在建立
└────┬────┘
     │
     ▼
┌─────────┐
│ SLEEPING │ ← 等待事件（檔案/記憶體）
└────┬────┘
     │
     ▼
┌─────────┐
│ RUNNABLE │ ← 可執行（等待 CPU）
└────┬────┘
     │
     ▼
┌─────────┐
│ RUNNING  │ ← 正在執行
└────┬────┘
     │
     ▼
┌─────────┐
│ ZOMBIE   │ ← 已結束，等待 wait
└─────────┘
```

## getenv/setenv

```c
char *getenv(const char *name);
int setenv(const char *name, const char *value, int overwrite);
int unsetenv(const char *name);
```

## 環境變數繼承

fork 後子程序繼承環境：
```c
setenv("MYVAR", "value", 1);
fork();  // 子程序也有 MYVAR=value
```

exec 後環境通常保持：
```c
setenv("MYVAR", "value", 1);
exec("/bin/env", ...);  // env 看到 MYVAR=value
```

## 使用者 ID

```c
uid_t getuid(void);     // 實際使用者 ID
uid_t geteuid(void);   // 有效使用者 ID
int setuid(uid_t uid);
```

## xv8 特殊程序

| PID | 程序 | 說明 |
|-----|------|------|
| 0 | swapper | 排程器程序（內核執行緒）|
| 1 | init | 第一個使用者程序 |

## 常見問題

### Zombie 程序的產生

```c
if (fork() == 0) {
    exit(0);  // 子程序終止
}
// 父程序未調用 wait
// 子程序變成 zombie
```

### Orphan 程序

父程序先於子程序終止：
- 子程序被 init (PID 1) 收養
- init 會呼叫 wait 回收

## 相關主題

- [[Process]]：xv8 的 Proc 結構
- [[Scheduler]]：程序排程
- [[File-Descriptor]]：fd 繼承