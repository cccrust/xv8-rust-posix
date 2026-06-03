# line — 行編輯器

提供支援歷史記錄的命令列編輯功能。

## 結構

```rust
pub struct LineEditor<'a> {
    buf: [u8; LineEditor::LINE_MAX],     // 目前行緩衝區
    len: usize,                          // 目前行長度
    cursor: usize,                       // 游標位置
    prompt: &'a str,                      // 提示字串

    history: [[u8; LineEditor::LINE_MAX]; 16],  // 歷史記錄
    history_lens: [usize; 16],
    history_entries: usize,
    history_offset: usize,
    stashed_buf: [u8; 256],               // 暫存目前行
    stashed_len: usize,
}
```

## 常數

```rust
const LINE_MAX: usize = 256;     // 最大行長度
const HISTORY_SIZE: usize = 16;  // 歷史記錄數量
```

## 主要 API

```rust
pub fn new() -> Self
pub fn read_line(&mut self, prompt: &str) -> Option<&str>
```

## 讀取行流程

```rust
pub fn read_line(&mut self, prompt: &str) -> Option<&str> {
    // 1. 設定 raw mode
    ioctl(Fd::STDIN, Ioctl::CONSOLE_SET_RAW, 1)?;

    // 2. 顯示提示並讀取按鍵
    loop {
        Stdin.read_exact(&mut c)?;
        match c[0] {
            b'\n' | b'\r' => break,  // Enter
            b'\x08' | b'\x7f' => self.backspace(),
            b'\x1b' => self.handle_escape(),  // ESC 序列
            // ... 其他按鍵處理
        }
    }

    // 3. 恢復 cooked mode
    ioctl(Fd::STDIN, Ioctl::CONSOLE_SET_RAW, 0)?;

    // 4. 加入歷史記錄
    if self.len > 0 {
        self.add_to_history();
    }

    Some(unsafe { str::from_utf8_unchecked(&self.buf[..self.len]) })
}
```

## 按鍵處理

| 按鍵 | 動作 |
|------|------|
| `Enter` / `Return` | 確認輸入 |
| `Backspace` / `Delete` | 刪除游標前字元 |
| `Ctrl-A` | 移到行首 |
| `Ctrl-E` | 移到行尾 |
| `Ctrl-U` | 刪除整行 |
| `Ctrl-W` | 刪除游標前單字 |
| `Ctrl-L` | 清屏 |
| `Ctrl-C` | 中斷輸入 |
| `Ctrl-D` | EOF（空行時） |
| `↑` | 歷史上一條 |
| `↓` | 歷史下一條 |
| `←` | 游標左移 |
| `→` | 游標右移 |

## ESC 序列處理

```rust
fn handle_escape(&mut self) {
    let mut seq = [0u8; 2];
    Stdin.read_exact(&mut seq).unwrap();

    match seq {
        [b'[', b'A'] => self.history_up(),    // ↑
        [b'[', b'B'] => self.history_down(),  // ↓
        [b'[', b'D'] => self.move_left(),     // ←
        [b'[', b'C'] => self.move_right(),    // →
        _ => {}
    }
}
```

## 游標操作

```rust
fn insert(&mut self, c: u8) {
    // 在游標位置插入字元
    for i in (self.cursor..self.len).rev() {
        self.buf[i + 1] = self.buf[i];
    }
    self.buf[self.cursor] = c;
    self.cursor += 1;
    self.len += 1;
    self.redraw();
}
```

## 歷史記錄

```rust
fn add_to_history(&mut self) {
    let slot = self.history_entries % HISTORY_SIZE;
    self.history[slot][..self.len].copy_from_slice(&self.buf[..self.len]);
    self.history_lens[slot] = self.len;
    self.history_entries += 1;
}

fn history_up(&mut self) {
    // 暫存目前行，載入歷史
    if self.history_offset == 0 {
        self.stashed_buf[..self.len].copy_from_slice(&self.buf[..self.len]);
        self.stashed_len = self.len;
    }
    self.history_offset += 1;
    self.load_from_history();
    self.redraw();
}
```

## 重繪

```rust
fn redraw(&self) {
    // "\r" + prompt + buf + "\x1b[K" + 移動游標
    out[n] = b'\r';
    // ... prompt ...
    out[n..n + self.len].copy_from_slice(&self.buf[..self.len]);
    // 清除行尾
    out[n..n + 3].copy_from_slice(b"\x1b[K");
    // 移動游標到正確位置
    let back = self.len - self.cursor;
    if back > 0 {
        out[n] = b'\x1b';
        // ... 發送游標移動序列 ...
    }
}
```

## 使用範例

```rust
use user::LineEditor;

let mut editor = LineEditor::new();
if let Some(line) = editor.read_line("xv8> ") {
    println!("You entered: {}", line);
}
```

## 與 shell 的整合

sh 使用 LineEditor 讀取使用者輸入：
```rust
let mut editor = LineEditor::new();
if let Some(line) = editor.read_line("posix> ") {
    // 剖析並執行命令
}
```

## 限制

- 不支援多行編輯
- 不支援自動補全
- ESC 序列僅支援基本方向鍵

## 相關主題

- [[io]]：I/O 操作
- [[sh]]：shell 整合