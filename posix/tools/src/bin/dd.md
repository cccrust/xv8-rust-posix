# dd — 資料複製和轉換

`dd`（data duplicator）是一個強大的資料複製和轉換工具，專門用於低層級的資料處理。

## 核心設計

`dd` 的特點是精確控制資料讀寫的各個參數：

```rust
let mut bs: u64 = 512;     // 區塊大小
let mut count: Option<u64> = None;  // 複製多少區塊
let mut seek_out: u64 = 0;  // 輸出檔案的起始位置
let mut skip_in: u64 = 0;   // 輸入檔案的跳過量
```

## 參數語法

`dd` 使用特殊的參數語法（`if`、`of`、`bs` 等）：

```rust
while i < args.len() {
    if args[i] == "if" && i + 1 < args.len() {
        ifile = Some(args[i + 1].clone());
        i += 2;
    } else if args[i] == "bs" && i + 1 < args.len() {
        bs = args[i + 1].parse().unwrap_or(512);
        i += 2;
    }
    // ...
}
```

- `if`（input file）：輸入檔案
- `of`（output file）：輸出檔案
- `bs`（block size）：區塊大小
- `count`：複製的區塊數
- `skip`：從輸入開頭跳過的區塊
- `seek`：從輸出開頭跳過的區塊

## 讀取和寫入

```rust
let mut input: Box<dyn Read> = match ifile {
    Some(ref path) => Box::new(File::open(path)?),
    None => Box::new(io::stdin().lock()),
};

let mut output: Box<dyn Write> = match ofile {
    Some(ref path) => Box::new(File::create(path)?),
    None => Box::new(io::stdout().lock()),
};
```

支援 stdin/stdout（預設）。

## 跳過輸入

```rust
if skip_in > 0 {
    let mut skipped = 0u64;
    while skipped < skip_in {
        let n = input.read(&mut buf[..(skip_in - skipped).min(4096) as usize])?;
        if n == 0 { break; }
        skipped += n as u64;
    }
}
```

`skip` 在讀取前跳過指定的位元組數。

## 跳過輸出

```rust
if seek_out > 0 {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::Start(seek_out))?;
}
```

`seek` 在寫入前移動到指定的位元組位置。

## 轉換

```rust
let lcase = conv == "lcase";
let ucase = conv == "ucase";

if lcase { data.make_ascii_lowercase(); }
if ucase { data.make_ascii_uppercase(); }
```

支援的轉換：
- `lcase`：轉換為小寫
- `ucase`：轉換為大寫

## 複製迴圈

```rust
loop {
    let n = input.read(&mut buf).unwrap_or(0);
    if n == 0 || blocks >= max_blocks { break; }
    let mut data = buf[..n].to_vec();
    if lcase { data.make_ascii_lowercase(); }
    output.write_all(&data)?;
    total += n as u64;
    blocks += 1;
}
```

每次讀取 `bs` 位元組，處理後寫出。

## 典型用途

### 複製檔案
```bash
dd if=/dev/sda of=/dev/sdb bs=4M status=progress
```

### 建立映像檔
```bash
dd if=/dev/cdrom of=cdrom.iso bs=2M
```

### 抹除磁碟
```bash
dd if=/dev/zero of=/dev/sdX bs=1M
```

### 轉換行結尾（Unix → DOS）
```bash
dd if=unix.txt of=dos.txt conv=ucase
```

### 擷取開頭
```bash
dd if=file of=first100bytes bs=1 count=100
```

### 備份 MBR
```bash
dd if=/dev/sda of=mbr.bin bs=512 count=1
```

## 輸出報告

```bash
eprintln!("{} bytes copied", total);
```

`dd` 會報告複製的位元組數（輸出到 stderr）。

## 與 cp 的比較

| 特性 | `dd` | `cp` |
|------|------|------|
| 精確控制 | 是 | 否 |
| 區塊大小 | 可調 | 自動優化 |
| 跳過/定位 | 支援 | 不直接支援 |
| 資料轉換 | 支援 | 有限 |

## 安全性

`dd` 直接操作硬碟/分割區，是非常危險的工具：
- 錯誤的 `of` 可能永久損壞資料
- 通常需要 root 權限
- 沒有確認提示

## 底層系統呼叫

- `open/read/write/seek/close`：基本的檔案 I/O
- 無需特殊的系統呼叫

## 效能考量

- `bs` 越大通常越快（減少系統呼叫）
- `iflag=direct` 避開緩衝
- `oflag=direct` 直接寫入磁碟

## 進度顯示

```bash
dd if=source of=dest bs=4M status=progress
```

`status=progress`（GNU dd）在複製時顯示進度。

## 實用範例

```bash
# 完整複製分割區
dd if=/dev/sda1 of=/dev/sdb1 bs=4M conv=noerror,sync

# 修復損壞的檔案系統
dd if=/dev/sda of=/dev/null bs=512 count=1

# 測試讀取速度
dd if=/dev/sda of=/dev/null bs=1M count=100 iflag=direct
```

## 相關指令

- `cp`：一般檔案複製
- `tr`：字元轉換
- `sdd`：`dd` 的圖形介面