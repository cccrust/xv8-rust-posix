# TTY — 終端控制

TTY (Teletypewriter) 是終端模擬設備，提供輸入輸出和程序控制。

## 層次結構

```
應用程式
   │
   │ read/write
   ▼
終端驅動（kernel）
   │
   │ 線路規則 (line discipline)
   ▼
終端設備 (/dev/tty, /dev/pts/0)
   │
   ▼
硬體或軟體終端模擬
```

## 終端類型

| 類型 | 說明 | 範例 |
|------|------|------|
| 主控台 | 直接連接的終端 | /dev/console |
| 虛擬終端 | 軟體模擬 | /dev/tty1-6 |
| PTY | 偽終端 | /dev/pts/0 |
| 序列埠 | 序列埠 | /dev/ttyS0 |

## isatty — 檢查是否為 TTY

```c
int isatty(int fd);
// 如果 fd 連接到終端返回 1，否則 0
```

```c
if (isatty(STDOUT_FILENO)) {
    printf("Output is a terminal\n");
} else {
    printf("Output is redirected or piped\n");
}
```

## ttyname — 取得終端名稱

```c
char *ttyname(int fd);
// 返回靜態緩衝區，NULL 如果不是終端
```

```c
printf("Terminal: %s\n", ttyname(STDIN_FILENO));
```

## termios — 終端屬性

```c
#include <termios.h>

struct termios {
    tcflag_t c_iflag;  // 輸入模式
    tcflag_t c_oflag;  // 輸出模式
    tcflag_t c_cflag;  // 控制模式
    tcflag_t c_lflag;  // 本地模式
    cc_t c_cc[NCCS];   // 控制字元
};
```

### tcgetattr/tcsetattr

```c
struct termios old, new;
tcgetattr(STDIN_FILENO, &old);

new = old;
new.c_lflag &= ~ECHO;  // 關閉回顯
tcsetattr(STDIN_FILENO, TCSANOW, &new);
```

## 常用模式

### 原始模式 (Raw)

```c
struct termios raw;
tcgetattr(fd, &raw);
cfmakeraw(&raw);  // 設定原始模式
tcsetattr(fd, TCSAFLUSH, &raw);
```

原始模式特點：
- 輸入立即可用（無行緩衝）
- 無自動回顯
- 無換行轉換

### 規範模式 (Canonical)

```c
struct termios cooked;
tcgetattr(fd, &cooked);
cooked.c_lflag |= ICANON;  // 開啟規範模式
tcsetattr(fd, TCSANOW, &cooked);
```

規範模式：
- 行緩衝（收到換行才返回）
- 自動回顯
- 標準控制字元（Ctrl+C, Ctrl+Z 等）

## 控制字元

```c
c_cc[VINTR] = 3;    // Ctrl+C = SIGINT
c_cc[VEOF] = 4;     // Ctrl+D = EOF
c_cc[VSUSP] = 26;   // Ctrl+Z = SIGTSTP
c_cc[VERASE] = 127; // DEL = erase
```

## tcflow — 流量控制

```c
tcflow(int fd, int action);
// TCOFF: 暫停輸出
// TCOON: 恢復輸出
// TCION: 暫停輸入
// TCIOFF: 發送 STOP 字元
```

## tcflush — 刷新緩衝

```c
tcflush(int fd, int queue);
// TCIFLUSH: 刷新輸入
// TCOFLUSH: 刷新輸出
// TCIOFLUSH: 刷新兩者
```

## stty — 設定終端屬性

```c
stty size        // 顯示大小
stty -echo       // 關閉回顯
stty echo        // 開啟回顯
```

## 訊號產生

終端產生的訊號：

| 按鍵 | 訊號 | 預設動作 |
|------|------|----------|
| Ctrl+C | SIGINT | 終止 |
| Ctrl+Z | SIGTSTP | 暫停 |
| Ctrl+\ | SIGQUIT | 傾印核心 |
| Ctrl+S | STOP | 暫停輸出 |
| Ctrl+Q | START | 恢復輸出 |

## 工作控制

```c
pid_t tcgetpgrp(int fd);           // 前景程序組
int tcsetpgrp(int fd, pid_t pgrpid); // 設定前景程序組
```

## 視窗大小

```c
struct winsize {
    unsigned short ws_row;     // 行數
    unsigned short ws_col;     // 列數
    unsigned short ws_xpixel;  // 像素（水平）
    unsigned short ws_ypixel;  // 像素（垂直）
};

ioctl(fd, TIOCGWINSZ, &winsize);
```

## 範例：密碼輸入

```c
struct termios old, new;
tcgetattr(STDIN_FILENO, &old);
new = old;
new.c_lflag &= ~ECHO;  // 關閉回顯
tcsetattr(STDIN_FILENO, TCSAFLUSH, &new);

printf("Password: ");
char pass[100];
fgets(pass, sizeof(pass), stdin);
pass[strcspn(pass, "\n")] = 0;

tcsetattr(STDIN_FILENO, TCSAFLUSH, &old);  // 恢復
```

## TIOCSTI

```c
ioctl(fd, TIOCSTI, "a");  // 模擬輸入字元 'a'
```

## 與 xv8 的關係

xv8 的 UART 驅動模擬簡單的終端介面。

## 安全性考量

- 終端設定可能影響安全性
- 攻擊者可能操縱終端設定
- 使用後應恢復原始設定

## 範例：Readline 風格編輯

```c
char buf[1024];
int pos = 0;

while (1) {
    char c = getchar();
    if (c == '\n') {
        buf[pos] = '\0';
        break;
    } else if (c == 127) {  // DEL
        if (pos > 0) {
            pos--;
            printf("\b \b");  // 擦除
        }
    } else {
        buf[pos++] = c;
        putchar(c);
    }
}
```

## 相關主題

- [[Signal]]：終端產生的訊號
- [[Process]]：工作控制