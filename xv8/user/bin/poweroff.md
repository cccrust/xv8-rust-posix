# Poweroff — 系統關機命令

`poweroff` 是 xv8 的系統關機命令，提供使用者空間觸發系統關機的方式。

## 實作方式

`poweroff` 透過系統呼叫通知核心終止所有行程並關閉系統。在 RISC-V QEMU 環境中，關機通常透過以下方式達成：

1. 寫入 QEMU 的 `poweroff` 裝置（通常為 `0x100000` 位址的 goldfish 或 `sifive_test` 裝置）觸發 QEMU 模擬器終止
2. 呼叫 OpenSBI 的系統關機 ecall（若 SBI 支援）

## 使用場景

在自動化測試中，testrunner 在所有測試通過後呼叫 `poweroff` 讓 QEMU 優雅退出，避免強制終止模擬器。這使測試框架可正確判斷測試成功或失敗。

## 相關文件

- [init.md](./init.md) — Init 行程
- [testrunner.md](../testbin/testrunner.md) — 測試執行器
