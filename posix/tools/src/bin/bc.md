# bc — 任意精度計算機

`bc`（Bench Calculator）是一種任意精度的數值計算語言，支援整數和浮點數運算。

## 核心設計

`bc` 使用逆波蘭表示法（RPN）和直譯器模式：

```rust
fn main() {
    let mut vars: HashMap<String, String> = HashMap::new();
    let scale: usize = 0;
    let ibase: u32 = 10;
    let obase: u32 = 10;

    if args.len() > 1 {
        let expr = args[1..].join(" ");
        if let Some(result) = eval_line(&expr, &mut vars, scale, ibase, obase) {
            println!("{}", result);
        }
    } else {
        // 互動模式
        for line in stdin.lock().lines() {
            let result = eval_line(&line, &mut vars, scale, ibase, obase);
            if let Some(r) = result { println!("{}", r); }
        }
    }
}
```

## 詞彙分析

```rust
fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut num = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' {
            num.push(c);
        } else {
            if !num.is_empty() { tokens.push(num.clone()); num.clear(); }
            if !c.is_whitespace() { tokens.push(c.to_string()); }
        }
    }
    tokens
}
```

## 逆波蘭表示法（Shunting Yard）

將中綴表達式轉換為後綴：

```rust
fn shunt(tokens: &[String]) -> Vec<String> {
    let mut rpn = Vec::new();
    let mut ops = Vec::new();
    // 運算子優先順序：+ - 低於 * / %
    // 括號優先於任何運算子
    // ...
}
```

## RPN 求值

```rust
fn eval_rpn(rpn: &[String]) -> Option<i64> {
    let mut stack: Vec<i64> = Vec::new();
    for token in rpn {
        match token.as_str() {
            "+" => { let b = stack.pop()?; let a = stack.pop()?; stack.push(a + b); }
            "-" => { let b = stack.pop()?; let a = stack.pop()?; stack.push(a - b); }
            "*" => { let b = stack.pop()?; let a = stack.pop()?; stack.push(a * b); }
            "/" => { let b = stack.pop()?; let a = stack.pop()?; stack.push(a / b); }
            n => { stack.push(n.parse().ok()?); }
        }
    }
    stack.pop()
}
```

## 變數支援

```rust
fn eval_arith(expr: &str, vars: &HashMap<String, String>) -> Option<String> {
    let mut s = expr.to_string();
    for (name, val) in vars.iter() {
        s = s.replace(name.as_str(), val.as_str());
    }
    let tokens = tokenize(&s);
    let rpn = shunt(&tokens);
    let result = eval_rpn(&rpn);
    result.map(|n| n.to_string())
}
```

變數在求值前先替換為其值。

## 輸出基數

```rust
if obase != 10 {
    return match obase {
        16 => Some(format!("{:X}", n_int)),
        8 => Some(format!("{:o}", n_int)),
        2 => Some(format!("{:b}", n_int)),
        _ => result,
    };
}
```

`obase` 控制輸出基數（2-16）。

## Scale（小數位數）

完整的 `bc` 有 `scale` 變數控制小數精度。xv8 的實現較簡化。

## 典型用途

```bash
# 互動模式
bc
# 輸入：3 + 4
# 輸出：7

# 單行計算
echo "3 + 4" | bc
bc <<< "3 + 4"

# 複利計算
echo "scale=2; 1000 * (1 + 0.05) ^ 10" | bc
```

## 與 dc 的比較

- `bc`：高階語法，類似 C
- `dc`：更低階，逆波蘭表示法更純粹

## 標準輸入的互動

```bash
bc -q  # 安靜模式（不輸出版本資訊）
bc -l  # 使用數學函式庫（s, c, a, l, e, j）
```

## 數學函式庫

完整 `bc` 支援：
- `s(x)`：正弦
- `c(x)`：餘弦
- `a(x)`：反正切
- `l(x)`：自然對數
- `e(x)`：指數
- `j(n, x)`：第一類貝索函式

## POSIX 規範

POSIX 定義了 `bc` 的基本行為和語法。

## 底層系統呼叫

`bc` 本身是使用者空間計算，不直接依賴系統呼叫。

## 相關指令

- `dc`：逆波蘭表示法計算機
- `expr`：整數表達式
- `awk`：文字處理和計算