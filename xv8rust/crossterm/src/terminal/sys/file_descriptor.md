# 終端檔案描述子輔助

提供取得終端檔案描述子的輔助函式。在 Unix 上透過 `/dev/tty` 或
標準錯誤（STDERR_FILENO）取得終端裝置的檔案描述子，用於 `tcgetattr`、
`ioctl` 等系統呼叫。此模組封裝了何時使用 stdout 與 stderr 的策略——
通常選擇 stderr 以避免管道操作時的干擾。
