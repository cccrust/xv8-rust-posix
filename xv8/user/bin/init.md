# init — 第一個使用者程序

init 是 xv8 系統啟動後的第一個使用者程序，負責啟動 shell 或測試執行器。

## 職責

1. 設定環境變數
2. 建立 console 裝置節點
3. 設定標準輸入/輸出/錯誤
4. 啟動 shell 或測試執行器

## 程式碼解析

```rust
fn main(_args: Args) {
    // 設定 PWD 環境變數
    let _ = setenv("PWD", "/", true);

    // 確保 console 裝置存在
    if open("console", OpenFlag::READ_WRITE).is_err() {
        mknod("console", CONSOLE, 0).expect("init: cannot create console");
        open("console", OpenFlag::READ_WRITE).expect("init: cannot open console");
    }

    // 設定標準輸入/輸出/錯誤
    dup(Fd::STDIN).expect("init: dup stdout");   // stdout → fd 0
    dup(Fd::STDIN).expect("init: dup stderr");   // stderr → fd 0

    // 檢查是否為測試模式
    let test_mode = open("testmode", OpenFlag::READ_ONLY).map(close).is_ok();

    // 主循環： fork 並執行 shell
    loop {
        let Ok(pid) = fork() else {
            exit_with_msg("init: fork failed");
        };

        if pid == 0 {
            if !test_mode {
                exec("/sh", &["sh"]);
                exit_with_msg("init: exec sh failed");
            } else {
                exec("/_testrunner", &["testrunner"]);
                exit_with_msg("init: exec testrunner failed");
            }
        }

        // 等待 shell 退出後重啟
        loop {
            let wpid = wait(&mut 0);
            if let Ok(wpid) = wpid {
                if wpid == pid {
                    break;  // shell 退出，重啟
                }
            }
        }
    }
}
```

## 檔案描述符設定

```
開啟後:
fd 0: stdin (console)
fd 1: stdout (console, dup from stdin)
fd 2: stderr (console, dup from stdin)
```

## 測試模式

如果存在 `/testmode` 檔案，init 會執行 `/_testrunner` 而非 `/sh`。這用於 QEMU 整合測試。

## 與 xv8 核心的整合

init 由 `proc::user_init()` 在核心初始化期間啟動，是第一個使用者程序。

## 相關主題

- [[sh]]：Unix shell
- [[Process]]：程序管理