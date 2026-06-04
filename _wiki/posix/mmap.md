# mmap — 記憶體映射

mmap 將檔案或裝置映射到記憶體位址空間。

## 基本語法

```c
void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
// addr: 建議位址（通常 NULL = 由核心選擇）
// length: 映射大小
// prot: 保護（PROT_READ/PROT_WRITE/PROT_EXEC）
// flags: MAP_SHARED/MAP_PRIVATE/MAP_ANONYMOUS
// fd: 檔案描述符（或 -1）
// offset: 檔案內偏移
```

## 檔案映射

```c
int fd = open("file.dat", O_RDONLY);
char *data = mmap(NULL, 4096, PROT_READ, MAP_SHARED, fd, 0);

// 現在可以像讀取陣列一樣讀取檔案
printf("%c", data[0]);

munmap(data, 4096);
close(fd);
```

## 寫入映射檔案

```c
int fd = open("file.dat", O_RDWR);
char *data = mmap(NULL, 4096, PROT_READ|PROT_WRITE, MAP_SHARED, fd, 0);

data[0] = 'A';  // 直接寫入，作業系統寫回檔案

msync(data, 4096, MS_SYNC);  // 確保寫入磁碟
munmap(data, 4096);
```

## 匿名映射

不與檔案關聯的映射（用於記憶體配置）：

```c
// 配置 4096 位元組，內容為零
char *buf = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                 MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
```

相當於 malloc，但保證映射是 page-aligned。

## MAP_SHARED vs MAP_PRIVATE

| 模式 | 行為 |
|------|------|
| MAP_SHARED | 修改寫回檔案/底層映射 |
| MAP_PRIVATE | 副本私有修改，不影響原始 |

```c
// MAP_PRIVATE：修改不影響檔案
char *data = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                 MAP_PRIVATE, fd, 0);
data[0] = 'X';  // 檔案不變
```

## brk/sbrk（歷史）

較旧的記憶體配置方式：

```c
void *sbrk(intptr_t increment);
// sbrk(0) 返回目前 break
// sbrk(4096) 增加 4096 位元組堆積
```

xv8 的 sbrk 實作：
```rust
pub fn sys_sbrk(n: usize) -> isize {
    let proc = current_proc();
    let new_size = (proc.data.size as isize + n as isize) as usize;
    proc.vm.alloc(proc.data.size, new_size, PTE_W)
}
```

## munmap — 解除映射

```c
int munmap(void *addr, size_t length);
```

## mprotect — 改變保護

```c
int mprotect(void *addr, size_t len, int prot);
```

```c
char *buf = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                 MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);

// 只讀
mprotect(buf, 4096, PROT_READ);

// 恢復寫入
mprotect(buf, 4096, PROT_READ|PROT_WRITE);
```

## msync — 同步到磁碟

```c
int msync(void *addr, size_t length, int flags);
// MS_ASYNC: 非同步寫入
// MS_SYNC: 同步寫入（等待完成）
// MS_INVALIDATE: 失效快取
```

## mremap — 重新映射

```c
void *mremap(void *old_addr, size_t old_size,
             size_t new_size, int flags);
// 用於擴展現有映射
```

## 用途

### 1. 動態載入

```c
// 將程式庫映射到記憶體
void *lib = mmap(NULL, size, PROT_READ|PROT_EXEC,
                 MAP_PRIVATE, fd, 0);
```

### 2. 共享記憶體

```c
// 兩個程序映射同一檔案
// 程式 A
char *shared = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                    MAP_SHARED, fd, 0);
shared[0] = 42;

// 程式 B
char *shared = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                    MAP_SHARED, fd, 0);
printf("%d", shared[0]);  // 42
```

### 3. 記憶體配置

```c
// 大塊連續記憶體
void *buf = mmap(NULL, 1024*1024, PROT_READ|PROT_WRITE,
                 MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
```

## 頁對齊

mmap 的 addr 和 offset 必須是頁大小（4KB）的倍數。

```rust
pub const PGSHIFT: usize = 12;
pub const PGSIZE: usize = 1 << PGSHIFT;  // 4096
```

## 錯誤處理

```c
void *ptr = mmap(NULL, size, PROT_READ, MAP_SHARED, fd, 0);
if (ptr == MAP_FAILED) {
    perror("mmap");
    exit(1);
}
```

注意：mmap 失敗返回 MAP_FAILED（(-1)），不是 NULL。

## 與其他系統的差異

| 特性 | POSIX | xv8 |
|------|-------|-----|
| MAP_ANONYMOUS | 有 | 有 |
| MAP_SHARED | 有 | 有 |
| mprotect | 有 | 有 |
| msync | 有 | 有限 |
| mremap | 有 | 無 |

## 效能考量

- 頁是記憶體管理的最小單位
- 首次訪問觸發 page fault
- 大檔案映射可能節省記憶體

## 限制

- 映射大小通常有限制
- addr 建議可能被忽略
- 檔案映射需要足夠的 fd

## 相關主題

- [[Virtual-Memory]]：分頁機制
- [[Process]]：記憶體佈局
- [[File-System]]：檔案 I/O