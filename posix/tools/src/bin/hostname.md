# hostname — 顯示或設定主機名

`hostname` 用於顯示或設定系統的主機名稱。

## 核心設計

```rust
let hostname = std::env::var("HOSTNAME")
    .or_else(|_| std::env::var("HOST"))
    .unwrap_or_else(|_| {
        #[cfg(unix)]
        {
            let mut buf = [0i8; 256];
            unsafe {
                if libc::gethostname(buf.as_mut_ptr(), buf.len()) == 0 {
                    return CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string();
                }
            }
        }
        "localhost".to_string()
    });
```

`hostname` 按順序嘗試：
1. 環境變數 `HOSTNAME`
2. 環境變數 `HOST`
3. 系統呼叫 `gethostname()`
4. 回退到 `localhost`

## gethostname 系統呼叫

```c
int gethostname(char *name, size_t len);
```

取得主機名稱，儲存到指定緩衝區。

## 主機名的用途

主機名用於：
- 網路識別（DNS）
- 電子郵件系統
- 系統日誌
- 網路服務綁定

## 三種主機名

完整的主機系統有三種名稱：

1. **靜態主機名**（static）：儲存在 `/etc/hostname`
2. **瞬態主機名**（transient）：內核維護，可被網路修改
3. **別名主機名**（pretty）：用於展示，可含特殊字元

```bash
hostnamectl
Static hostname: myserver
Pretty hostname: My Server
Icon name: computer
Chassis: vm
Machine ID: abc123...
Boot ID: def456...
```

## DNS 主機名 vs 網域名稱

- **主機名（hostname）**：區域網路內的識別
- **FQDN（Fully Qualified Domain Name）**：完整網域名稱

```bash
hostname           # 輸出：myserver
hostname -f        # 輸出：myserver.example.com
```

## 典型用途

```bash
# 顯示主機名
hostname

# 設定主機名（需要 root）
hostname newname

# 顯示完整網域名稱
hostname -f
```

## 環境變數

某些系統依賴環境變數：
- `HOSTNAME`：廣泛支援
- `HOST`：備用

## 網路環境中的主機名

在 DHCP 環境中，主機名可能由 DHCP 伺服器動態分配。

## /etc/hostname

Linux 系統的靜態主機名儲存在此檔案，開機時讀取。

## 跨平台

```rust
#[cfg(unix)]
{
    unsafe { libc::gethostname(buf.as_mut_ptr(), buf.len()) }
}

#[cfg(not(unix))]
{
    "localhost".to_string()
}
```

Unix 上使用 `gethostname()`，其他系統有不同方式。

## 主機名限制

- 長度通常限制在 64 或 255 字元
- 可包含字母、數字、連字符
- 不能以連字符開頭
- 不能全是數字

## 相關檔案

- `/etc/hostname`：靜態主機名
- `/etc/hosts`：主機名解析（本地 DNS）
- `/proc/sys/kernel/hostname`：瞬態主機名

## 設定主機名

Linux 上設定主機名：
```bash
# 臨時設定（立即生效）
hostname newname

# 永久設定（Red Hat）
hostnamectl set-hostname newname

# 永久設定（Debian）
echo "newname" > /etc/hostname
```

## 安全性

主機名通常公開在網路上，注意：
- 不要在主機名中包含敏感資訊
- 避免使用真實的企業/個人名稱

## 底層系統呼叫

- `gethostname()`：取得主機名
- `sethostname()`：設定主機名（需要 CAP_SYS_ADMIN）

## 實用範例

```bash
# 在網頁伺服器中
<VirtualHost *:80>
    ServerName hostname
    ServerAlias www.hostname
</VirtualHost>

# 在 SSH 配置中
Host remote
    HostName hostname.example.com
    User admin
```

## 相關指令

- `hostnamectl`：systemd 主機名管理
- `uname -n`：顯示節點名稱（等價於 `hostname`）
- `domainname`：顯示/設定 NIS 網域名稱
- `dnsdomainname`：顯示 DNS 網域名稱