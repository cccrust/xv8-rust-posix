# Primes — 質數計算展示

`primes` 是一個計算質數的展示程式，在 xv8 上展示 CPU 運算能力與行程間通訊。

## 演算法背景

### 埃拉托斯特尼篩法（Sieve of Eratosthenes）

最經典的質數產生演算法：

1. 建立從 2 到 N 的整數列表
2. 從最小的質數（2）開始，標記其倍數為合數
3. 取下一個未標記的數字，重複步驟 2

### 管線（Pipeline）版本

xv8 的 `primes` 可能使用 Doug McIlroy 提出的經典管道篩法：每個行程從管道讀取數字，找到自己的質數，將非倍數寫入下一個管道。這形成了 Unix pipe 機制的典型教學範例。

## 展示要點

- CPU 整數運算效能
- pipe 行程間通訊
- 多行程協同合作

## 相關文件

- [pipe.md](../testbin/pipe.md) — Pipe 測試
- [proc.md](../../kernel/src/proc.md) — 行程管理
