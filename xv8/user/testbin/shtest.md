# Shtest — Shell 行為測試

`shtest` 測試驗證 xv8 的 POSIX shell（sh）在 RISC-V 上的行為正確性。測試涵蓋 93 項 shell 行為檢測，包括變數展開、命令替換、管道、重定向、條件判斷、迴圈、函數定義、工作控制等。這些測試確保 xv8 的 shell 實作符合 POSIX.1-2017 Shell & Utilities 卷規範。

## 相關文件

- [sh.md](../../../posix/tools/src/bin/sh.md) — Shell 實作
- [testrunner.md](./testrunner.md) — 測試框架
- [shtest.sh](../../../xv8/shtest.sh) — Shell 測試腳本
