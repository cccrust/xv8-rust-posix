# pwd — 顯示目前目錄

`pwd`（print working directory）顯示 shell 目前的工作目錄絕對路徑。

## 核心設計

```rust
fn main() {
    match std::env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => {
            eprintln!("pwd: {}", e);
            std::process::exit(1);
        }
    }
}
```

`pwd` 使用 Rust 的 `std::env::current_dir()` 取得目前工作目錄。

## 工作目錄（Working Directory）

工作目錄是 shell 當前所在的檔案系統位置，相對路徑都基於此目錄。

## 符號連結 vs 實際路徑

完整的 `pwd` 有兩個模式：

```bash
pwd        # 顯示實際路徑
pwd -P     # 顯示邏輯路徑（解析符號連結後）
```

## shell 內建 vs 獨立程式

在多數 shell 中，`pwd` 是內建命令：
- 內建 `pwd`：讀取 shell 內部狀態（`$PWD` 變數）
- `/bin/pwd`：使用系統呼叫讀取

兩者通常行為相同，但 shell 內建更快。

## 環境變數

Shell 通常維護 `PWD` 環境變數：
```bash
echo $PWD
```

## 典型用途

```bash
# 確認目前位置
pwd

# 在指令中使用
cp file.txt $(pwd)/backup/

# 與絕對路徑結合
cd $(pwd)/subdir
```

## 與 cd 的關係

`cd` 改變工作目錄，`pwd` 顯示工作目錄。

## 路徑表示

`pwd` 輸出絕對路徑：
- 絕對路徑：以 `/` 開頭
- 相對路徑：以 `./` 或 `../` 開頭

```bash
/home/user/projects  # 絕對路徑
./subdir            # 相對路徑（基於目前目錄）
../parent           # 上層目錄
```

## 底層系統呼叫

`pwd` 使用 `getcwd()` 系統呼叫：

```c
char *getcwd(char *buf, size_t size);
```

Linux 的實現在 `glibc` 中。

## 錯誤處理

```rust
Err(e) => {
    eprintln!("pwd: {}", e);
    std::process::exit(1);
}
```

常見錯誤：
- `ENOENT`：目錄被刪除
- `ERANGE`：路徑太長（緩衝區不夠）

## 安全性考量

`pwd` 不需要特殊權限，任何使用者都可以執行。

## 實用範例

```bash
# 基本使用
pwd

# 確定在哪個目錄
ls -la $(pwd)

# 確認 cd 的結果
cd /tmp && pwd
```

## 相關指令

- `cd`：變更目錄
- `pushd`/`popd`：目錄堆疊
- ` dirs`：顯示目錄堆疊