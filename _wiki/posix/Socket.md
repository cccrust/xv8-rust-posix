# Socket — 網路通訊端

Socket 是程序間網路通訊的端點。

## 概述

```
程序 A                              程序 B
   │                                    ▲
   │  socket()                         │
   │     │                             │
   │     ▼                             │
   │  bind() ──────── 位址綁定          │
   │     │                             │
   │     ▼                             │
   │  listen()    ──── 開始監聽         │
   │     │                             │
   │     ▼                             │
   │  accept() ◄────── 連接到來         │
   │     │                             │
   │     ▼                             │
   ├───────────────────────────────────┤
   │  send() / recv()                  │
   │  write() / read()                 │
   ├───────────────────────────────────┤
   │                                    │
   ▼                                    │
close() ◄──────────────────────────────┘
```

## socket 建立

```c
int socket(int domain, int type, int protocol);
// domain:   AF_INET (IPv4), AF_INET6 (IPv6), AF_UNIX (本機)
// type:    SOCK_STREAM (TCP), SOCK_DGRAM (UDP), SOCK_RAW
// protocol: 0 (自動選擇)

// TCP socket
int sock = socket(AF_INET, SOCK_STREAM, 0);

// UDP socket
int sock = socket(AF_INET, SOCK_DGRAM, 0);
```

## bind — 綁定位址

```c
struct sockaddr_in {
    sa_family_t sin_family;  // AF_INET
    in_port_t   sin_port;    // 連接埠號
    struct in_addr sin_addr; // IP 位址
};

struct sockaddr_in addr;
addr.sin_family = AF_INET;
addr.sin_port = htons(8080);
addr.sin_addr.s_addr = INADDR_ANY;  // 任何介面

bind(sock, (struct sockaddr *)&addr, sizeof(addr));
```

## listen — 監聽

```c
int listen(int sockfd, int backlog);
// backlog: 連接佇列長度
```

## accept — 接受連接

```c
int client = accept(sockfd, struct sockaddr *addr, socklen_t *addrlen);
// 堵塞直到客戶端連接
// 返回新的 socket 描述符
```

## connect — 連接

```c
struct sockaddr_in server;
server.sin_family = AF_INET;
server.sin_port = htons(80);
inet_pton(AF_INET, "93.184.216.34", &server.sin_addr);

connect(sock, (struct sockaddr *)&server, sizeof(server));
```

## send/recv — 訊息傳遞

```c
ssize_t send(int sockfd, const void *buf, size_t len, int flags);
ssize_t recv(int sockfd, void *buf, size_t len, int flags);

// flags: MSG_OOB, MSG_PEEK, MSG_DONTWAIT
```

## write/read — 通用 I/O

Socket 也是檔案描述符，可用 read/write：

```c
write(sockfd, "hello", 5);
read(sockfd, buf, sizeof(buf));
```

## sendto/recvfrom — UDP

```c
ssize_t sendto(int sockfd, const void *buf, size_t len, int flags,
               const struct sockaddr *dest_addr, socklen_t addrlen);
ssize_t recvfrom(int sockfd, void *buf, size_t len, int flags,
                 struct sockaddr *src_addr, socklen_t *addrlen);
```

## close — 關閉

```c
close(sockfd);
```

## 位址結構

### IPv4

```c
struct in_addr {
    uint32_t s_addr;  // 網路位元組序
};

struct sockaddr_in {
    sa_family_t sin_family;
    in_port_t   sin_port;
    struct in_addr sin_addr;
};
```

### IPv6

```c
struct in6_addr {
    uint8_t s6_addr[16];
};

struct sockaddr_in6 {
    sa_family_t sin6_family;
    in_port_t   sin6_port;
    uint32_t    sin6_flowinfo;
    struct in6_addr sin6_addr;
    uint32_t    sin6_scope_id;
};
```

## 位址轉換

```c
// 字串 → 二進位
inet_pton(AF_INET, "192.168.1.1", &addr.sin_addr);

// 二進位 → 字串
char buf[INET_ADDRSTRLEN];
inet_ntop(AF_INET, &addr.sin_addr, buf, sizeof(buf));
```

## 主機位元組序 vs 網路位元組序

```c
uint16_t htons(uint16_t x);  // 主機 → 網路 (short)
uint16_t ntohs(uint16_t x);  // 網路 → 主機 (short)
uint32_t htonl(uint32_t x);  // 主機 → 網路 (long)
uint32_t ntohl(uint32_t x);  // 網路 → 主機 (long)
```

## getsockname / getpeername

```c
struct sockaddr_in local, remote;
socklen_t len = sizeof(local);

getsockname(sockfd, (struct sockaddr *)&local, &len);
getpeername(sockfd, (struct sockaddr *)&remote, &len);
```

## 範例：TCP 伺服器

```c
int main() {
    int sock = socket(AF_INET, SOCK_STREAM, 0);

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(8080),
        .sin_addr.s_addr = INADDR_ANY
    };
    bind(sock, (struct sockaddr *)&addr, sizeof(addr));
    listen(sock, 10);

    while (1) {
        int client = accept(sock, NULL, NULL);
        char buf[1024];
        int n = read(client, buf, sizeof(buf));
        write(client, buf, n);
        close(client);
    }
}
```

## 範例：TCP 客戶端

```c
int main() {
    int sock = socket(AF_INET, SOCK_STREAM, 0);

    struct sockaddr_in server = {
        .sin_family = AF_INET,
        .sin_port = htons(8080),
    };
    inet_pton(AF_INET, "127.0.0.1", &server.sin_addr);

    connect(sock, (struct sockaddr *)&server, sizeof(server));
    write(sock, "hello", 5);

    char buf[100];
    int n = read(sock, buf, sizeof(buf));
    close(sock);
}
```

## 範例：UDP 收發

```c
// 接收
struct sockaddr_in from;
socklen_t fromlen = sizeof(from);
char buf[1024];
int n = recvfrom(sockfd, buf, sizeof(buf), 0,
                (struct sockaddr *)&from, &fromlen);

// 發送
sendto(sockfd, buf, n, 0, (struct sockaddr *)&from, fromlen);
```

## setsockopt

```c
int setsockopt(int sockfd, int level, int optname,
              const void *optval, socklen_t optlen);

// 重用位址
int opt = 1;
setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

// 設定逾時
struct timeval tv;
tv.tv_sec = 5;
setsockopt(sockfd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
```

## shutdown

優雅關閉連接：

```c
int shutdown(int sockfd, int how);
// how: SHUT_RD (停止讀), SHUT_WR (停止寫), SHUT_RDWR (兩者)
```

## 非阻塞 I/O

```c
int flags = fcntl(sockfd, F_GETFL, 0);
fcntl(sockfd, F_SETFL, flags | O_NONBLOCK);
```

## select/poll

多路復用 I/O：

```c
fd_set readfds;
FD_ZERO(&readfds);
FD_SET(sockfd, &readfds);

struct timeval timeout = { .tv_sec = 5, .tv_usec = 0 };
select(sockfd + 1, &readfds, NULL, NULL, &timeout);
```

## Unix Domain Socket

本機程序間通訊：

```c
struct sockaddr_un {
    sa_family_t sun_family;
    char sun_path[108];  // 路徑名
};

// 伺服器
unlink("/tmp/socket");
int sock = socket(AF_UNIX, SOCK_STREAM, 0);
struct sockaddr_un addr = { .sun_family = AF_UNIX, .sun_path = "/tmp/socket" };
bind(sock, (struct sockaddr *)&addr, sizeof(addr));
listen(sock, 5);

// 客戶端
connect(sock, (struct sockaddr *)&addr, sizeof(addr));
```

## 與 xv8 的關係

xv8 有基本的網路堆疊（Ethernet/ARP/IP/UDP/ICMP），但 TCP 支援有限。

## 常見錯誤

| 錯誤 | 原因 |
|------|------|
| EADDRINUSE | 位址已被使用 |
| ECONNREFUSED | 拒絕連接 |
| ETIMEDOUT | 連接逾時 |
| ENOTCONN | 未連接（對已關閉的 socket 寫入）|

## 相關主題

- [[Network-Stack]]：xv8 網路堆疊
- [[Pipe]]：程序間通訊