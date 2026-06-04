# true — 返回成功

`true` 是最簡單的 Unix 命令之一，什麼都不做，只是返回成功（exit code 0）。

## 極簡實現

```rust
fn main() {
    std::process::exit(0);
}
```

就是這麼簡單。`true` 的唯一任務就是以 0（表示成功）的 exit code 退出。

## 用途

`true` 主要用於：
1. **Shell 腳本中的無操作**：當需要 placeholder
2. **無限循環**：`while true; do ...; done`
3. **預設返回成功**：在 case 語句中
4. **條件成立時的空操作**：`if condition; then true; fi`

## 與 `:（冒號）內建命令的比較

Shell 的內建命令 `:`（冒號）是另一個什麼都不做的命令：

```bash
:   # 等價於 true
```

兩者幾乎完全相同，但 `:` 是 shell 內建，不需要建立新程序。

## 作為預設命令

`true` 常用於提供預設值：

```bash
# 如果 VAR 未設定，使用 "default"
value=${VAR:-$(true; echo default)}

# 什麼都不做的迴圈
while true; do
    sleep 1
done
```

## 與 false 的對比

`true` → exit code 0（成功）
`false` → exit code 1（失敗）

```bash
if true; then echo "This always runs"; fi
if false; then echo "This never runs"; fi
```

## 測試框架中的用途

在測試腳本中，`true` 可用於確保某個分支總是成功：

```bash
test_condition && true || echo "Test failed"
```

## Shell 函數

在 Bash 中，`true` 也可以定義為函數：

```bash
true() { return 0; }
```

## 底層系統呼叫

`true` 幾乎不需要系統呼叫。`exit(0)` 直接調用 `exit_group` syscall。

## 標準相容性

`true` 必須：
- 返回 exit code 0
- 不輸出任何內容
- 不讀取任何輸入

POSIX 標準明確定義了 `true` 的行為。

## 相關指令

- `false`：返回失敗
- `:`（冒號）：Shell 內建的無操作命令
- `yes`：無限輸出 "y"
- `exit`：退出 shell