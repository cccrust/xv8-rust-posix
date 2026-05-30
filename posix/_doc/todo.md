# bash 修复进度

## 已修复

1. **skip_blank 无限循环** - line 23
   - 修复：添加 `if self.advance().is_none() { break; }`

2. **Iterator.next 递归** - line 117
   - 修复：改为 `Lexer::next(self)`

3. **变量 token 丢失 $** - line 56
   - 修复：添加 `v.push('$');`

4. **expand_vars 重写** - lines 137-228
   - 移除所有 `continue`，使用 if-else 链
   - 简化 $(( 和 $( 处理逻辑

5. **删除残留代码** - lines 229-301
   - 已删除重复的旧代码

## 待修复

### 1. `$((1+1))` 输出 `2) )`

**问题**：lexer 返回 token `"$((1+1))"` 少一个 `)`

**原因**：format 字符串 `$(({}))` 只有一个 `)`，但实际需要两个

**位置**：line 56
```rust
return Some(format!("$(({}))", expr)); // 当前：1个 )
```

**修复**：改成
```rust
return Some(format!("$(({}) ))", expr)); // 两个 )
```

### 2. `$(date)` 类似问题

**原因**：同上，format 需要两个 `)`

### 3. `$HOME` 展开为空

**原因**：环境变量没加载到 globals 中

## 测试命令

```bash
./target/debug/bash -c 'echo $((1+1))'  # 期望: 2
./target/debug/bash -c 'x=5; echo $x'  # 期望: 5
./target/debug/bash -c 'echo $(date +%H)'  # 期望: 当前小时
./target/debug/bash -c 'echo hello world'  # 期望: hello world
```