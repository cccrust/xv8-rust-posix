# Environment — 環境變數

環境變數是程序繼承的鍵值對字串集合。

## 概述

```
login
  │
  ▼
初始化環境（HOME, USER, SHELL...）
  │
  ▼
fork + exec
  │
  ▼
子程序繼承環境
```

## getenv/setenv

```c
char *getenv(const char *name);
// 返回值或 NULL

int setenv(const char *name, const char *value, int overwrite);
// overwrite: 0=不覆蓋, 1=覆蓋
```

## unsetenv

```c
int unsetenv(const char *name);
// 移除環境變數
```

## clearenv

```c
int clearenv(void);
// 清空環境（xv8 有）
```

## 常用環境變數

| 變數 | 說明 | 範例 |
|------|------|------|
| PATH | 指令搜尋路徑 | `/bin:/usr/bin` |
| HOME | 家目錄 | `/home/user` |
| USER | 使用者名稱 | `john` |
| SHELL | 登入 shell | `/bin/sh` |
| PWD | 目前目錄 | `/home/user` |
| TERM | 終端類型 | `xterm-256color` |
| LANG | 語言/地區 | `en_US.UTF-8` |
| LD_LIBRARY_PATH | 動態連結庫路徑 | `/usr/local/lib` |

## 環境與 exec

exec 預設繼承環境：

```c
setenv("MYVAR", "hello", 1);
exec("/bin/myprogram");  // myprogram 收到 MYVAR
```

execve 第三個參數可以指定新環境：

```c
char *envp[] = { "PATH=/bin", "HOME=/root", NULL };
execve("/bin/program", argv, envp);  // 使用新環境
```

## 環境與 fork

fork 後子程序繼承父程序的環境：

```c
setenv("PARENT", "1", 1);
if (fork() == 0) {
    // 子程序也有 PARENT=1
    // 對 setenv 的修改不會影響父程序
}
```

## C 語言的 main 參數

```c
int main(int argc, char *argv[], char *envp[]);
// argc: 參數數量
// argv: 參數陣列
// envp: 環境變數陣列（NULL 結尾）
```

```c
int main(int argc, char *argv[], char *envp[]) {
    // argv[0] = 程式名稱
    // argv[1] = 第一個參數
    // ...
    // argv[argc] = NULL

    // envp[0] = "PATH=/bin"
    // envp[1] = "HOME=/root"
    // envp[n] = NULL

    for (char **e = envp; *e; e++) {
        printf("%s\n", *e);
    }
}
```

## getenv vs environ

```c
extern char **environ;

printf("%s\n", getenv("PATH"));
printf("%s\n", environ[0]);  // 相同
```

## 環境變數污染

惡意環境變數可能影響程式行為：

```bash
# LD_PRELOAD 攻擊
LD_PRELOAD=/tmp/malicious.so ./program

# PATH 毒化
PATH=/tmp:$PATH ls  # 執行假的 ls
```

## 常見模式

### 設定並執行

```c
setenv("PATH", "/usr/local/bin:/usr/bin:/bin", 1);
execlp("program", "program", NULL);
```

### 讀取並修改

```c
char *path = getenv("PATH");
char *newpath = malloc(strlen(path) + 20);
sprintf(newpath, "%s:/new/path", path);
setenv("PATH", newpath, 1);
free(newpath);
```

## 區域設定

LC_ALL 和 LC_* 變數控制地區：

```bash
LC_ALL=en_US.UTF-8
LC_TIME=zh_TW.UTF-8
LANG=en_US.UTF-8
```

## 實驗場景

xv8 的 `env` 工具：

```bash
env              # 顯示所有環境變數
env VAR=value cmd # 設定並執行
env -i cmd       # 清空環境執行
```

## 與 xv6 的差異

| 函式 | xv6 | xv8 |
|------|-----|-----|
| getenv | 無 | 有 |
| setenv | 無 | 有 |
| unsetenv | 無 | 有 |
| clearenv | 無 | 有 |
| environ | 有 | 有 |

## procfs 介面

Linux 的 `/proc/self/environ` 顯示環境：

```bash
cat /proc/self/environ | tr '\0' '\n'
```

## 常見問題

### 記憶體洩漏

```c
// 錯誤：每次 setenv 都分配新記憶錄
while (condition) {
    setenv("VAR", value, 1);  // 每次分配
}
// 正確：只用一次
setenv("VAR", value, 1);
```

### 執行緒安全

getenv 不是執行緒安全的：
- 多執行緒同時讀寫可能出問題
- 解決：使用區域變數儲存結果

## 相關主題

- [[Process]]：程序與環境
- [[exec]]：程式執行與環境繼承