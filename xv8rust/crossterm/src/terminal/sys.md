# 終端系統模組分派器

根據平台分派終端操作：尺寸查詢、原始模式設定、畫面清除、替代緩衝區切換等。
Unix 使用 `unix` 模組的 termios 操作，Windows 使用 `windows` 模組的 Console API，
RISC-V 使用 `riscv64` stub。此分派器確保所有終端層級操作具有統一的抽象介面，
隱藏 `ioctl`、`SetConsoleMode` 等平台差異。
