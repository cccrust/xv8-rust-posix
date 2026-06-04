# tsort — 拓撲排序

`tsort` 對有向圖進行拓撲排序。

## 核心設計

```rust
let pairs: Vec<(String, String)> = lines.chunks(2)
    .filter(|c| c.len() == 2)
    .map(|c| (c[0].clone(), c[1].clone()))
    .collect();
```

輸入為「誰在誰前面」的配對：`A B` 表示 A 在 B 前面。

## 建立圖結構

```rust
let mut adj: HashMap<String, Vec<String>> = HashMap::new();
let mut in_deg: HashMap<String, usize> = HashMap::new();

for (a, b) in &pairs {
    adj.entry(a.clone()).or_default().push(b.clone());
    in_deg.entry(a.clone()).or_insert(0);
    *in_deg.entry(b.clone()).or_insert(0) += 1;
}
```

- `adj`：鄰接表（A → B 表示 A 在 B 前面）
- `in_deg`：入度（每個節點有多少個前置節點）

## Kahn 演算法

```rust
let mut queue: Vec<String> = in_deg.iter()
    .filter(|(_, &deg)| deg == 0)
    .map(|(k, _)| k.clone())
    .collect();

while let Some(node) = queue.pop() {
    result.push(node.clone());
    if let Some(neighbors) = adj.remove(&node) {
        for n in neighbors {
            if let Some(deg) = in_deg.get_mut(&n) {
                *deg -= 1;
                if *deg == 0 { queue.push(n.clone()); }
            }
        }
    }
}
```

Kahn 演算法：
1. 找出入度為 0 的節點（無前置要求）
2. 輸出該節點，移除其所有邊
3. 重複直到隊列為空

## 圖的表示

```
輸入：  a b
       b c
       c d

圖：    a → b → c → d
       └───────────┘

拓撲排序：a b c d
```

## 輸入格式

```
element1 element2
```

每行兩個元素，表示第一個在第二個之前。

## 典型用途

### 建構系統依賴
```bash
# 編譯順序
# A依賴B，B依賴C，C依賴D
# D C B A
echo -e "B A\nC B\nD C" | tsort
```

### 解析依賴
```bash
# 軟體包安裝順序
dpkg --info package.dep 2>/dev/null | grep -i depends | tr ',' '\n' | ...
```

## 循環檢測

如果有循環，拓撲排序無法完成：
```
A B
B A
```

`tsort` 會輸出部分結果（遇到的節點），其餘的順序是任意的。

## 與其他排序的比較

| 工具 | 用途 |
|------|------|
| `tsort` | 拓撲排序（依賴圖）|
| `sort` | 字母排序 |
| `comm` | 集合交集/差集 |

## 輸出

```
a
b
c
d
```

每行一個元素。

## 底層系統呼叫

`tsort` 使用：
- `read()`：讀取輸入
- HashMap：記憶體內圖結構

## 實用範例

```bash
# 簡單依賴
echo -e "a b\nb c" | tsort

# 複雜依賴
echo -e "d c\nc b\nb a" | tsort
# 輸出：d c b a

# 檔案
tsort dependencies.txt
```

## 演算法複雜度

- 時間：O(V + E)
- 空間：O(V + E)

V = 節點數，E = 邊數

## 與 make

`make` 內部使用拓撲排序確定編譯順序。

## 相關指令

- `tac`：反轉行順序
- `sort`：排序
- `make`：建構工具