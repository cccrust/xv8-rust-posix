# args — 命令列參數處理

提供使用者程式的命令列參數解析功能。

## 核心結構

### Args

```rust
pub struct Args {
    argc: usize,          // 參數個數（含程式名稱）
    argv: *const *const u8,  // 參數指標陣列
}
```

從 RISC-V a0/a1 暫存器初始化：

```rust
pub unsafe fn from_stack() -> Self {
    asm!(
        "mv {0}, a0",  // argc
        "mv {1}, a1",  // argv
        out(reg) argc,
        out(reg) argv,
    )
}
```

### ArgsIter

```rust
pub struct ArgsIter {
    argv: *const *const u8,
    current: usize,
    end: usize,
}
```

迭代器，逐一走訪參數。

## 讀取參數

```rust
// 取得參數個數
args.len()           // 含程式名稱
args.args_len()      // 不含程式名稱

// 取得指定索引的參數
args.get(0)                      // 程式名稱 (byte slice)
args.get_str(0)                  // 程式名稱 (&str)
args.get(1)                      // 第一個參數
args.get_str(2)                  // 第二個參數

// 迭代
for arg in args.iter() { }       // 含程式名稱
for arg in args.args() { }       // 不含程式名稱
for arg in args.iter_as_str() { } // as &str
for arg in args.args_as_str() { } // as &str, 不含程式名
```

## 使用範例

```rust
fn main(args: Args) {
    // 程式名稱
    let program = args.program();

    // 取得特定參數
    if args.len() >= 2 {
        let input = args.get_str(1).unwrap();
        // ...
    }

    // 迭代所有參數（不含程式名）
    for arg in args.args_as_str() {
        println!("{}", arg);
    }
}
```

## 指標操作

```rust
// 取得指標
let ptr = *self.argv.add(index);

// 讀取 C 字串
let mut len = 0;
while *ptr.add(len) != 0 {
    len += 1;
}

// 轉換為 slice
slice::from_raw_parts(ptr, len)
```

## 安全注意事項

- `from_stack()` 必須在程式開始時呼叫
- 參數指標指向核心記憶體，使用者不可修改
- 回傳的 `&'static [u8]` 和 `&'static str` 在程序結束前有效

## 轉換工具

```rust
// C 字串轉 Rust str
unsafe fn str_from_cstr<'a>(cstr: &[u8]) -> Result<&'a str, Utf8Error> {
    let mut len = 0;
    while len < cstr.len() && *ptr.add(len) != 0 {
        len += 1;
    }
    str::from_utf8(slice::from_raw_parts(ptr, len))
}
```

## 與 exec 的整合

當 `exec()` 成功執行新程式時，Args 會從新程式的 stack取得。

## 相關主題

- [[syscall]]：系統呼叫