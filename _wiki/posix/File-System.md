# File-System — 檔案系統 API

POSIX 定義的標準檔案系統介面。

## open — 開啟檔案

```c
int open(const char *pathname, int flags, ... /* mode_t mode */);
// flags: O_RDONLY, O_WRONLY, O_RDWR
//       O_CREAT, O_EXCL, O_TRUNC, O_APPEND
// mode: 檔案權限（建立時）
```

### 範例

```c
int fd = open("/tmp/test", O_WRONLY|O_CREAT|O_TRUNC, 0644);
```

## close — 關閉

```c
int close(int fd);
```

關閉後 fd 可被重用。

## read — 讀取

```c
ssize_t read(int fd, void *buf, size_t count);
// 返回讀取的位元組數，0 表示 EOF，-1 表示錯誤
```

## write — 寫入

```c
ssize_t write(int fd, const void *buf, size_t count);
```

## lseek — 移動檔案指標

```c
off_t lseek(int fd, off_t offset, int whence);
// whence: SEEK_SET(從頭), SEEK_CUR(從目前), SEEK_END(從尾)
```

## creat — 建立檔案（歷史）

```c
int creat(const char *pathname, mode_t mode);
// 等價於 open(pathname, O_WRONLY|O_CREAT|O_TRUNC, mode)
```

## unlink — 刪除

```c
int unlink(const char *pathname);
```

減少連結計數，檔案內容實際刪除在計數為 0 時。

## rename — 重新命名

```c
int rename(const char *oldpath, const char *newpath);
```

## mkdir/rmdir

```c
int mkdir(const char *pathname, mode_t mode);
int rmdir(const char *pathname);  // 目錄必須為空
```

## opendir/closedir/readdir

```c
DIR *dir = opendir("/tmp");
struct dirent *entry;
while ((entry = readdir(dir)) != NULL) {
    printf("%s\n", entry->d_name);
}
closedir(dir);
```

## stat/fstat/lstat

```c
struct stat {
    dev_t     st_dev;      // 裝置
    ino_t     st_ino;      // inode 號碼
    mode_t    st_mode;     // 檔案類型和權限
    nlink_t   st_nlink;    // 硬連結數
    uid_t     st_uid;      // 擁有者
    gid_t     st_gid;      // 群組
    off_t     st_size;     // 大小
    time_t    st_atime;    // 最後存取
    time_t    st_mtime;    // 最後修改
    time_t    st_ctime;    // 最後狀態變更
};

int stat(const char *pathname, struct stat *statbuf);
int fstat(int fd, struct stat *statbuf);
int lstat(const char *pathname, struct stat *statbuf);  // 不跟隨 symlink
```

## access — 檢查權限

```c
int access(const char *pathname, int mode);
// mode: R_OK, W_OK, X_OK, F_OK
```

## chmod/fchmod

```c
int chmod(const char *path, mode_t mode);
int fchmod(int fd, mode_t mode);
```

## link/symlink

```c
int link(const char *oldpath, const char *newpath);   // 硬連結
int symlink(const char *target, const char *linkpath); // 符號連結
```

## readlink — 讀取連結目標

```c
ssize_t readlink(const char *path, char *buf, size_t bufsiz);
```

## 檔案類型判斷

```c
struct stat st;
stat(path, &st);

if (S_ISREG(st.st_mode))   // 一般檔案
if (S_ISDIR(st.st_mode))    // 目錄
if (S_ISLNK(st.st_mode))    // 符號連結
if (S_ISCHR(st.st_mode))   // 字元裝置
if (S_ISBLK(st.st_mode))   // 區塊裝置
if (S_ISFIFO(st.st_mode))  // FIFO
if (S_ISSOCK(st.st_mode))  // Socket
```

## sync/fsync

```c
int sync(void);     // 同步所有檔案系統
int fsync(int fd);  // 同步特定檔案
```

## truncate/ftruncate

```c
int truncate(const char *path, off_t length);
int ftruncate(int fd, off_t length);
```

## 檔案描述符操作

```c
int dup(int oldfd);              // 複製到最小 fd
int dup2(int oldfd, int newfd);  // 複製到指定 fd
int fcntl(int fd, int cmd, ...); // 各種控制
```

## ioctl — 設備控制

```c
int ioctl(int fd, unsigned long request, ...);
```

用於終端、網路卡等特殊控制。

## 範例：複製檔案

```c
int src = open("src.txt", O_RDONLY);
int dst = open("dst.txt", O_WRONLY|O_CREAT|O_TRUNC, 0644);

char buf[8192];
ssize_t n;
while ((n = read(src, buf, sizeof(buf))) > 0) {
    write(dst, buf, n);
}

close(src);
close(dst);
```

## 錯誤處理

```c
if (open("/tmp/test", O_RDONLY) < 0) {
    perror("open failed");
    exit(1);
}
```

## 與 xv8 的對應

| POSIX | xv8 | 說明 |
|-------|-----|------|
| open | sys_open | 開啟檔案 |
| read | sys_read | 讀取 |
| write | sys_write | 寫入 |
| close | sys_close | 關閉 |
| link | sys_link | 硬連結 |
| unlink | sys_unlink | 刪除 |
| mkdir | sys_mkdir | 建立目錄 |

## 相關主題

- [[File-Descriptor]]：fd 機制
- [[File-System]]：xv8 內部實作
- [[Pipe]]：匿名管道