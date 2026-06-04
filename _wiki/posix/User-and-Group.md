# User-and-Group — 使用者與群組

POSIX 的使用者識別和存取控制。

## UID 和 GID 類型

| 類型 | 說明 | 範圍 |
|------|------|------|
| UID | 使用者 ID | 0-65534 |
| GID | 群組 ID | 0-65534 |
| EUID | 有效 UID（權限檢查用）| |
| EGID | 有效 GID | |
| RUID | 實際 UID（登入者）| |
| RGID | 實際 GID | |

## 基本函式

```c
uid_t getuid(void);      // 實際 UID
gid_t getgid(void);      // 實際 GID
uid_t geteuid(void);    // 有效 UID
gid_t getegid(void);    // 有效 GID
```

## getpwuid/getpwnam

取得使用者資訊：

```c
struct passwd {
    char *pw_name;   // 登入名稱
    uid_t pw_uid;   // UID
    gid_t pw_gid;   // GID
    char *pw_dir;   // 家目錄
    char *pw_shell; // 登入 shell
};

struct passwd *getpwuid(uid_t uid);
struct passwd *getpwnam(const char *name);
```

```c
struct passwd *pw = getpwuid(getuid());
printf("User: %s, Home: %s\n", pw->pw_name, pw->pw_dir);
```

## getgrgid/getgrnam

取得群組資訊：

```c
struct group {
    char *gr_name;   // 群組名
    gid_t gr_gid;    // GID
    char **gr_mem;   // 成員列表
};

struct group *grp = getgrgid(gid);
```

## setuid/setreuid

```c
int setuid(uid_t uid);  // 設定 UID
int seteuid(uid_t uid); // 設定有效 UID
int setgid(gid_t gid);
int setegid(gid_t gid);
```

### 特權行為

- root (UID=0) 可以設定任何 UID
- 普通用戶只能設定為自己的 RUID 或 EUID

## setreuid/setregid

設定實際和有效 UID：

```c
int setreuid(uid_t ruid, uid_t euid);
// -1 表示不改變
```

## 群組設定

```c
int setgroups(size_t size, const gid_t *list);
int initgroups(const char *user, gid_t basegid);
```

## getgroups

```c
int getgroups(int size, gid_t list[]);
// 返回所屬群組數量
```

## chown — 改變擁有者

```c
int chown(const char *path, uid_t owner, gid_t group);

chown("file", 1000, 1000);    // 設定 owner 和 group
chown("file", 1000, -1);       // 只改變 owner
chown("file", -1, 1000);       // 只改變 group
```

## chmod — 改變權限

```c
int chmod(const char *path, mode_t mode);

// 數值形式
chmod("file", 0644);  // rw-r--r--

// 符號形式需要 parse
chmod("file", u+rw);  // 增加擁有者讀寫
```

## umask — 檔案建立遮罩

```c
mode_t umask(mode_t mask);
// 返回先前的 mask

umask(0022);  // 新檔案權限 = 0666 & ~0022 = 0644
```

## 檔案權限結構

```
-rwxr-xr--  1 user group  1234  Jan  1 10:00  file
││││││││
││││││││└─ others: r--
│││││││└── group: r-x
││││││└─── owner: rwx
│││││└───── sticky, setuid, setgid
││││└─────── 其他權限
│││└──────── owner 權限
││└────────── group 權限
│└──────────── others 權限
└────────────── 檔案類型 (- = 的一般檔案)
```

## 特殊權限位

| 位 | 說明 | 對檔案 | 對目錄 |
|----|------|--------|--------|
| SUID | 執行時以擁有者身份 | 執行 | 無視 |
| SGID | 執行時以群組身份 | 執行 | 內新檔案繼承目錄的 group |
| Sticky | | 無視 | 只能刪除自己的檔案 |

```bash
chmod u+s file   # SUID
chmod g+s dir    # SGID
chmod +t dir     # Sticky
```

## getpwent — 列舉使用者

```c
struct passwd *p;
while ((p = getpwent()) != NULL) {
    printf("%s:%d\n", p->pw_name, p->pw_uid);
}
endpwent();
```

## getgrent — 列舉群組

```c
struct group *g;
while ((g = getgrent()) != NULL) {
    printf("%s:%d\n", g->gr_name, g->gr_gid);
}
endgrent();
```

## /etc/passwd 格式

```
root:x:0:0:root:/root:/bin/sh
user:x:1000:1000:User Name:/home/user:/bin/sh
```

## /etc/group 格式

```
root:x:0:
user:x:1000:user1,user2
```

## login

登入過程：
1. 取得使用者名稱
2. getpwnam 查詢密碼檔
3. 驗證密碼
4. initgroups 設定群組
5. setuid 切換身份
6. chdir 到家目錄
7. chroot（可選）
8. exec shell

## 安全性

### 檢查

- 使用有效 UID/GID
- 權限檢查總是使用 EUID/EGID
- 核心忽略 SUID 程序嘗試變回 root

### 陷阱

```c
// 危險：檢查的是 EUID，實際可能不同
if (geteuid() == 0) {
    // 看似特權
}
```

## 與 xv8 的關係

xv8 的使用者系統（部分）：

| 功能 | xv8 | 說明 |
|------|-----|------|
| getuid | 有 | |
| getgid | 有 | |
| getpwuid | 有限 | |
| chown | 基本 | |
| chmod | 基本 | |

## 常用 UID

| UID | 名稱 | 說明 |
|-----|------|------|
| 0 | root | 超級使用者 |
| 1 | daemon | 系統 daemon |
| 2 | bin | 系統工具 |
| 1000+ | user | 一般用戶 |

## 相關主題

- [[File-System]]：權限檢查
- [[Signal]]：作業控制