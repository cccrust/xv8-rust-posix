# Init — 第一個行程（PID 1）

`init` 是 xv8 啟動後核心執行的第一個使用者行程，PID 為 1。它是使用者空間的根行程，負責初始化使用者環境。

## Init 的職責

1. **系統初始化**：掛載虛擬檔案系統（procfs、sysfs）、設定網路介面、啟動必要的背景服務
2. **孤兒行程收養**：當父行程終止而子行程未結束時，子行程成為孤兒，被 init 收養並等待
3. **服務管理**：啟動 shell 或其他使用者介面供使用者操作

## 實作模式

xv8 的 init 在檢測到 `/tmp/testmode` 標記檔案時，會執行測試測試器（testrunner）而非互動式 shell。這讓 QEMU 測試流程可以自動化。

## 相關文件

- [demo.md](./demo.md) — 展示程式
- [testrunner.md](../testbin/testrunner.md) — 測試執行器
- [main.md](../../kernel/src/main.md) — 核心初始化
