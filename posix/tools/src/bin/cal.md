# cal — 顯示日曆

`cal` 顯示公曆日曆，是 Unix 經典工具之一。

## 核心設計

```rust
fn print_month(month: i32, year: i32) {
    let months = ["January","February","March","April","May","June","July","August","September","October","November","December"];
    let m = month.max(1).min(12) as usize;
    let title = format!("{} {}", months[m-1], year);
    println!("{:^20}", title);
    println!("Su Mo Tu We Th Fr Sa");

    let first = weekday(1, month, year);  // 1 號是星期幾
    let days = days_in_month(month, year);

    for _ in 0..first { print!("   "); }  // 開頭空白
    for d in 1..=days {
        print!("{:2} ", d);
        if (d as usize + first) % 7 == 0 { println!(); }
    }
}
```

## 星期計算（Zeller's Congruence）

```rust
fn weekday(day: i32, month: i32, year: i32) -> usize {
    // Zeller-like for Gregorian
    let (m, y) = if month < 3 { (month + 12, year - 1) } else { (month, year) };
    ((day + (13 * (m + 1)) / 5 + y % 100 + (y % 100) / 4 + (y / 100) / 4 - 2 * (y / 100)) % 7 + 7) as usize % 7
}
```

Zeller's Congruence 公式用於計算給定日期是星期幾：
- 週日 = 0, 週一 = 1, ..., 週六 = 6
- 需要特殊處理 1 月和 2 月（視為上一年 13、14 月）

## 每月天數

```rust
fn days_in_month(month: i32, year: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap(year) { 29 } else { 28 },
        _ => 0,
    }
}
```

月份天數查詢表，2 月需要考慮閏年。

## 閏年判斷

```rust
fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
```

閏年規則：
- 能被 4 整除
- 但不能被 100 整除
- 除非能被 400 整除

## 輸出格式

```
    January 2024
Su Mo Tu We Th Fr Sa
    1  2  3  4  5  6
 7  8  9 10 11 12 13
14 15 16 17 18 19 20
21 22 23 24 25 26 27
28 29 30 31
```

標題居中，星期標題佔 20 字元寬。

## 閏年的歷史

1582 年教皇 Gregory XIII 推行 Gregorian 曆法時，因為累積的誤差，直接跳過了 10 天（10/5-10/14）。

## 預設行為

不帶參數時，顯示**目前月份**：

```bash
cal  # 顯示目前月份的日曆
```

## 參數處理

```rust
if args.len() > 1 {
    if let Ok(y) = args[1].parse::<i32>() {
        print_year(y);  // 只有年份：顯示全年
        return;
    }
}
print_month(month, year);  // 預設顯示目前月份
```

支援的用法：
- `cal`：目前月份
- `cal 2024`：顯示 2024 年全年
- `cal 3 2024`：顯示 2024 年 3 月

## 與 ncal 的比較

`ncal`（或 `cal -n`）是以不同格式顯示：
- 垂直排列月份
- 顯示復活節日期等

## 星期開始

美國通常以週日開始（Su），歐洲以週一開始。

## 文化差異

不同地區的週起始日：
- 美國：週日（Su）
- ISO：週一（Mo）
- 猶太曆：週日（Sun）
- 伊斯蘭曆：週六

## 歷史背景

`cal` 首次出現在 Version 7 Unix，是 AT&T Bell Labs 的經典工具。

## 典型用途

```bash
# 顯示目前月份
cal

# 顯示年曆
cal 2024

# 顯示特定月份
cal 3 2024

# 查看某月的最後一天
cal 2 2024 | tail -2
```

## 與 date 的關係

`date` 處理時間，`cal` 處理日期：
- `date`：時間點
- `cal`：日曆視圖

## 實用範例

```bash
# 檢查某月有多少天
cal 2 2024 | tail -1 | awk '{print $NF}'

# 找某年某月第一天是星期幾
cal 6 2024 | head -3 | tail -1 | awk '{print $1}'
```

## 底層系統呼叫

`cal` 使用 `localtime_r` 或日曆計算，與 `date` 類似。

## 相關指令

- `date`：顯示時間
- `ncal`：垂直排列的日曆
- `calendar`：提醒工具（Unix 工具）