# Pipe — 管道

管道是程序間單向資料通道，一端寫入、另一端讀取。

## 建立管道

```c
int pipe(int fds[2]);
// fds[0] = 讀取端
// fds[1] = 寫入端
```

## 基本使用

```c
int p[2];
pipe(p);

// fork 前建立
if (fork() == 0) {
    // 子程序：關閉讀取端
    close(p[0]);
    write(p[1], "hello", 5);
    close(p[1]);
} else {
    // 父程序：關閉寫入端
    close(p[1]);
    char buf[100];
    int n = read(p[0], buf, sizeof(buf));
    close(p[0]);
}
```

## 資料流

```
程序 A                      程序 B
   │                           ▲
   │  pipe(p)                  │
   │  p[0] ◄─── read          │
   │  p[1] ───► write         │
   │                           │
   ▼                           │
```

## pipe 缓冲区

pipe 有有限緩衝區（通常 64KB）：

- 緩衝區滿時 write 阻塞
- 緩衝區空時 read 阻塞
- 所有 write 端關閉後 read 返回 0（EOF）

## 關閉末端

### 讀取端關閉

如果讀取端關閉，寫入端：
- write 到已關閉的讀取端 → SIGPIPE
- 預設 SIGPIPE 終止程式

### 寫入端關閉

如果寫入端關閉，讀取端：
- read 返回 0（EOF）

## 常見模式

### 連接 stdout 到 pipe

```c
int p[2];
pipe(p);

if (fork() == 0) {
    // 子程序
    close(p[0]);
    close(STDOUT_FILENO);  // 關閉 stdout
    dup(p[1]);              // dup 會使用最小可用 fd = 1
    close(p[1]);
    exec("ls");
}
```

### popen 实现

```c
FILE *popen(const char *cmd, const char *mode) {
    int p[2];
    pipe(p);

    if (fork() == 0) {
        if (*mode == 'r') {
            close(p[0]);
            close(STDOUT_FILENO);
            dup(p[1]);
        } else {
            close(p[1]);
            close(STDIN_FILENO);
            dup(p[0]);
        }
        execl("/bin/sh", "sh", "-c", cmd, NULL);
        _Exit(127);
    }

    if (*mode == 'r') {
        close(p[1]);
        return fdopen(p[0], "r");
    } else {
        close(p[0]);
        return fdopen(p[1], "w");
    }
}
```

## 管道族

### popen/pclose

```c
FILE *fp = popen("ls -la", "r");
char buf[256];
while (fgets(buf, sizeof(buf), fp) != NULL) {
    printf("%s", buf);
}
pclose(fp);
```

### mkfifo — 命名管道

建立有名稱的管道，可用於無關程序間通訊：

```c
mkfifo("/tmp/myfifo", 0666);

 // 程序 A（讀取）
int fd = open("/tmp/myfifo", O_RDONLY);
read(fd, buf, sizeof(buf));

// 程序 B（寫入）
int fd = open("/tmp/myfifo", O_WRONLY);
write(fd, "hello", 5);
```

## 阻塞特性

預設 pipe 操作是阻塞的：

```c
// 讀取會阻塞直到有資料或 EOF
n = read(p[0], buf, 100);

// 寫入會阻塞直到有空間
n = write(p[1], buf, 100);
```

## 非阻塞設定

```c
// 設定非阻塞
int flags = fcntl(p[0], F_GETFL);
fcntl(p[0], F_SETFL, flags | O_NONBLOCK);

read(p[0], buf, 100);  // 無資料時返回 -1，errno = EAGAIN
```

## 管道緩衝區大小

可以使用 fcntl 取得/設定：

```c
long size = fpathconf(p[0], _PC_PIPE_BUF);
```

## 原子性

寫入小於 PIPE_BUF 的資料是原子的（不會被分割）。

```c
#define PIPE_BUF 4096
write(p[1], buf, 100);  // 原子寫入
```

## 與 xv6 的差異

| 特性 | xv6 | xv8 |
|------|-----|-----|
| pipe() | 有 | 有 |
| pipe 緩衝區 | 512 bytes | 較大 |
| mkfifo | 無 | 基本 |

## 常見錯誤

| 錯誤 | 原因 |
|------|------|
| EMFILE | 檔案描述符耗盡 |
| ENFILE | 系統 pipe 表滿 |
| EPIPE | 讀取端已關閉 |

## 用途

- 程序通訊（IPC）
- 連接程式（`cmd1 | cmd2`）
- 程序池的任務分發
- 緩衝區 producer/consumer

## 限制

- 單向（雙向需兩個 pipe）
- 相關程序間（父子、兄弟）
- 有限緩衝區
- 無廣播（需要多個 pipe）

## 相關主題

- [[Process]]：fork 與 pipe 的結合
- [[File-Descriptor]]：fd 機制
- [[Shell]]：管道的 shell 實現