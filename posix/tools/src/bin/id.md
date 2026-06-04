# id — 顯示使用者和群組 ID

`id` 顯示目前使用者或指定使用者的 UID、GID 以及所屬群組。

## 核心設計

```rust
fn get_ids() -> (u32, u32, u32, u32) {
    unsafe {
        (libc::getuid(), libc::geteuid(), libc::getgid(), libc::getegid())
    }
}
```

四個 ID：
- `getuid()`：真實 UID（real UID）
- `geteuid()`：有效 UID（effective UID）
- `getgid()`：真實 GID
- `getegid()`：有效 GID

## UID 和 GID 的區別

### 真實 UID（ruid）
- 程序啟動時的 UID
- 通常是登入使用者的 UID
- 用於追蹤誰啟動了程序

### 有效 UID（euid）
- 用於權限檢查
- 可以透過 setuid 程式臨時提升
- 例如 `passwd` 程式以 root 身份執行，但 ruid 是普通用戶

## 顯示格式

```rust
println!("uid={}({}) gid={}({})", uid, user_name(uid), gid, group_name(gid));
```

預設輸出：
```
uid=1000(user) gid=1000(user) groups=1000(user),4(adm),27(sudo)
```

## 選項解析

```rust
match c {
    'G' => show_groups = true,   // 只顯示附屬群組
    'g' => show_group = true,    // 只顯示 GID
    'n' => show_name = true,    // 顯示名稱而非數字
    'r' => show_real = true,    // 顯示真實 ID
    'u' => show_user = true,    // 只顯示 UID
}
```

## -n 選項

```rust
let format_id = |id: u32| -> String {
    if show_name { user_name(id) } else { id.to_string() }
};
```

`-n` 讓 `id` 顯示名稱而非數字：
```bash
id -n
# uid=user gid=user

id -un
# user
```

## -r 選項

```rust
let use_real = show_real;
let uid = if use_real { ruid } else { euid };
let gid = if use_real { rgid } else { egid };
```

`-r` 顯示真實 UID/GID：
```bash
id -ur
# 1000
```

## 群組解析

```rust
fn group_name(gid: u32) -> String {
    unsafe {
        let gr = libc::getgrgid(gid);
        if !gr.is_null() {
            return CStr::from_ptr((*gr).gr_name).to_string_lossy().to_string();
        }
    }
    gid.to_string()
}
```

使用 `getgrgid()` 將 GID 解析為群組名。

## 附屬群組

```rust
unsafe {
    let ngroup = libc::getgroups(0, ptr::null_mut());
    if ngroup > 0 {
        let mut groups: Vec<gid_t> = vec![0; ngroup as usize];
        libc::getgroups(ngroup, groups.as_mut_ptr());
        // ...
    }
}
```

使用 `getgroups()` 取得附屬群組列表。

## 典型用途

```bash
# 顯示目前使用者
id

# 顯示使用者名（而非數字）
id -n

# 只顯示 UID
id -u

# 顯示附屬群組
id -G

# 以特定使用者身份執行（需配合 su）
su -c "id" username
```

## 與 whoami 的比較

- `id`：顯示完整的使用者和群組資訊
- `whoami`：只顯示使用者名稱（等價於 `id -un`）

## 系統呼叫

`id` 依賴多個系統呼叫：
- `getuid()`：取得真實 UID
- `geteuid()`：取得有效 UID
- `getgid()`：取得真實 GID
- `getegid()`：取得有效 GID
- `getpwuid()`：UID → 使用者名
- `getgrgid()`：GID → 群組名
- `getgroups()`：取得附屬群組

## 安全相關

`id` 可用於：
- 確認當前身份
- 確認 setuid 程式的效果
- 故障排除時確認權限

## 特殊 UID

- `0`：root（超級使用者）
- `65534`：nobody（通常用於 NFS）
- `65535`：有時用於無效 UID

## 實用範例

```bash
# 確認是否為 root
id -u  # 輸出 0 表示是 root

# 在腳本中檢查權限
if [ $(id -u) -eq 0 ]; then
    echo "Running as root"
fi
```

## 相關指令

- `whoami`：顯示使用者名
- `who`：顯示已登入使用者
- `groups`：顯示群組關係
- `newgrp`：切換主要群組