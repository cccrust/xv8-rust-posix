# Linux pidfd 包裝器

提供行程檔案描述子（pidfd）相關系統呼叫的 Rust 包裝。pidfd 是 Linux 5.3+
引入的機制，透過 `pidfd_open()` 取得代表行程的檔案描述子，可用於：
`pidfd_send_signal()`——向行程發送訊號而不受 PID 重用競爭影響；
`poll()`/`epoll()`——監控行程退出狀態（替代 `waitpid` 的非阻塞方案）；
`CLONE_PIDFD`——在 clone 時直接取得子行程的 pidfd。
pidfd 解決了傳統 PID 重用（PID reuse race）的安全問題，是現代 Linux 行程管理的重要基礎。
