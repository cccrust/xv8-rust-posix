# Time — 時間函式

POSIX 提供多種時間相關函式。

## 時間類型

### time_t

```c
typedef long time_t;  // 秒 since Epoch (1970-01-01 00:00:00 UTC)
```

### struct timeval

```c
struct timeval {
    time_t      tv_sec;   // 秒
    suseconds_t tv_usec;  // 微秒
};
```

### struct timespec

```c
struct timespec {
    time_t tv_sec;   // 秒
    long   tv_nsec;  // 奈秒
};
```

### struct tm

日曆時間結構：

```c
struct tm {
    int tm_sec;    // 秒 (0-60, 60 for leap seconds)
    int tm_min;    // 分鐘 (0-59)
    int tm_hour;   // 小時 (0-23)
    int tm_mday;   // 日期 (1-31)
    int tm_mon;    // 月份 (0-11)
    int tm_year;   // 年份 (since 1900)
    int tm_wday;   // 星期 (0-6, Sunday=0)
    int tm_yday;   // 一年中的第幾天 (0-365)
    int tm_isdst;  // 夏令時間旗標
};
```

## gettimeofday

```c
int gettimeofday(struct timeval *tv, struct timezone *tz);
// tv: 必須非 NULL
// tz: 可為 NULL
```

```c
struct timeval now;
gettimeofday(&now, NULL);
printf("%ld seconds since Epoch\n", now.tv_sec);
```

## clock_gettime

更精確的時間（建議使用）：

```c
int clock_gettime(clockid_t clock_id, struct timespec *tp);

// clock_id:
//   CLOCK_REALTIME  - 系統時鐘
//   CLOCK_MONOTONIC - 啟動後持續遞增
//   CLOCK_PROCESS_CPUTIME_ID - 程序的 CPU 時間
//   CLOCK_THREAD_CPUTIME_ID   - 執行緒的 CPU 時間
```

```c
struct timespec ts;
clock_gettime(CLOCK_MONOTONIC, &ts);
printf("%ld.%09ld seconds\n", ts.tv_sec, ts.tv_nsec);
```

## time

```c
time_t time(time_t *tloc);
// 如果 tloc 非 NULL，結果也存到 *tloc
```

```c
time_t now = time(NULL);
printf("%s", ctime(&now));
```

## ctime / ctime_r

將 time_t 轉為字串：

```c
char *ctime(const time_t *timep);
char *ctime_r(const time_t *timep, char *buf);

// "Wed Jun  3 10:00:00 2026\n"
```

## gmtime / localtime

將 time_t 轉為 tm 結構：

```c
struct tm *gmtime(const time_t *timep);       // UTC
struct tm *localtime(const time_t *timep);    // 本地時間
```

```c
time_t now = time(NULL);
struct tm *tm = localtime(&now);
printf("%d-%02d-%02d %02d:%02d:%02d\n",
    tm->tm_year + 1900,
    tm->tm_mon + 1,
    tm->tm_mday,
    tm->tm_hour,
    tm->tm_min,
    tm->tm_sec);
```

## strftime

格式化時間為字串：

```c
size_t strftime(char *s, size_t max, const char *format, const struct tm *tm);

char buf[100];
strftime(buf, sizeof(buf), "%Y-%m-%d %H:%M:%S", &tm);
printf("%s\n", buf);
```

### 格式說明符

| 說明符 | 說明 | 範例 |
|--------|------|------|
| %Y | 年（4 位）| 2026 |
| %m | 月（2 位）| 06 |
| %d | 日（2 位）| 03 |
| %H | 小時（2 位）| 10 |
| %M | 分鐘（2 位）| 00 |
| %S | 秒（2 位）| 00 |
| %a | 星期縮寫 | Wed |
| %A | 星期全名 | Wednesday |
| %b | 月份縮寫 | Jun |
| %B | 月份全名 | June |

## mktime

反向轉換：

```c
time_t mktime(struct tm *tm);
// tm 中的 tm_isdst 可設為 0（不使用）或 1（使用）
// -1 表示讓系統判斷
```

## sleep/usleep/nanosleep

```c
unsigned sleep(unsigned seconds);      // 秒
int usleep(useconds_t usec);           // 微秒（已廢棄）
int nanosleep(const struct timespec *req, struct timespec *rem);
```

```c
// 睡 1.5 秒
struct timespec req = { 1, 500000000 };
nanosleep(&req, NULL);
```

## alarm

```c
unsigned alarm(unsigned seconds);
// 設定計時器，seconds 秒後發送 SIGALRM
```

## 計時測量

```c
struct timespec start, end;

clock_gettime(CLOCK_MONOTONIC, &start);
// 要測量的程式碼
clock_gettime(CLOCK_MONOTONIC, &end);

double elapsed = (end.tv_sec - start.tv_sec) +
                 (end.tv_nsec - start.tv_nsec) / 1e9;
```

## 時區

```c
struct timezone {
    int tz_minuteswest;  // UTC 以西分鐘數
    int tz_dsttime;      // 夏令時間類型
};
```

通常設為 NULL。

## difftime

```c
double difftime(time_t time1, time_t time2);
// 返回 time1 - time2（秒）
```

## Epoch

1970 年 1 月 1 日 00:00:00 UTC 為 Unix Epoch。

```c
// Epoch + N 秒
time_t future = 86400 * 365;  // 一年後
time_t when = 0;  // Epoch
when += 86400;   // 一天後
when += 3600;    // 一小時後
```

## 與 xv8 的關係

xv8 的時間相關 syscall：

| 函式 | syscall | 說明 |
|------|---------|------|
| time | sys_gettimeofday | 取得目前時間 |
| sleep | sys_sleep | 睡眠指定 tick |
| alarm | sys_alarm | 設定 alarm |

## 常見用法

### 測量執行時間

```c
struct timespec start, end;
clock_gettime(CLOCK_MONOTONIC, &start);

do_work();

clock_gettime(CLOCK_MONOTONIC, &end);
double ms = (end.tv_sec - start.tv_sec) * 1000 +
            (end.tv_nsec - start.tv_nsec) / 1e6;
```

### Timeout

```c
struct timespec deadline;
clock_gettime(CLOCK_REALTIME, &deadline);
deadline.tv_sec += 5;  // 5 秒超時

while (condition) {
    // 做些事
    nanosleep(&interval, NULL);
    if (clock_gettime(CLOCK_REALTIME, &now) < 0) break;
    if (now.tv_sec > deadline.tv_sec) break;
}
```

## 相關主題

- [[Signal]]：SIGALRM
- [[Process]]：程序時間統計