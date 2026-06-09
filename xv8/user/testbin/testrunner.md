# Testrunner — 測試執行器

`testrunner` 是 xv8 的整合測試框架，在 QEMU 環境中負責自動化執行所有測試二進位檔。init 行程在 `/tmp/testmode` 存在時啟動 testrunner，後者依序執行所有測試程式並報告結果。每個測試由 `testrunner` fork/exec 執行，收集退出碼判斷通過或失敗。測試完成後呼叫 `poweroff` 關閉 QEMU。

## 相關文件

- [init.md](../bin/init.md) — Init 行程
- [poweroff.md](../bin/poweroff.md) — 系統關機
- [shtest.md](./shtest.md) — Shell 測試
