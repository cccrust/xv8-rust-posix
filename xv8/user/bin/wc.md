# wc — 字、行、位元組計數

wc 計算檔案中的行數、字數和位元組數。

## 使用方式

```bash
wc [file...]
```

## 輸出格式

```
 行數   字數   位元組  檔案
   12     45    256  file.txt
```

## 實作

```rust
fn wc(mut fd: Fd, name: &str) {
    let mut l = 0;  // 行數
    let mut w = 0;  // 字數
    let mut c = 0;  // 位元組
    let mut in_word = false;

    let mut buf = [0u8; 512];

    while let Ok(n) = fd.read(&mut buf) {
        if n == 0 {
            println!("{} {} {} {}", l, w, c, name);
            return;
        }

        // 計算
        let slice = if n < buf.len() {
            buf[n] = 0;
            unsafe { str_from_cstr(&buf) }.unwrap_or(&buf[..n])
        } else {
            &buf[..n]
        };

        c += slice.len();
        l += slice.chars().filter(|&c| c == '\n').count();
        w += slice.split_whitespace().count();
    }

    exit_with_msg("wc: read error");
}
```

## 字計數邏輯

```rust
// 追蹤是否在單字內
if in_word && str.starts_with(|c: char| !c.is_whitespace()) {
    w -= 1;  // 跨越緩衝區邊界的單字
}
in_word = str.ends_with(|c: char| !c.is_whitespace());
```

## 與 POSIX 的差異

- 不支援 `-l`（只顯示行數）
- 不支援 `-w`（只顯示字數）
- 不支援 `-c`（只顯示位元組）

## 範例

```bash
wc file.txt              # 計算單個檔案
wc f1.txt f2.txt         # 計算多個檔案
cat file.txt | wc        # 計算 stdin
```

## 相關主題

- [[cat]]：檔案輸出