# Awk — 模式掃描與處理語言

`awk` 是一種程式語言，專為文字檔案的模式掃描與處理設計，由 Aho、Weinberger 與 Kernighan 開發。`awk` 程式由 pattern {action} 規則組成，逐行處理輸入檔案，將每行分割為欄位（$0=整行，$1=第一欄）。內建變數包括 FS（欄位分隔符，預設為空白）、NR（行號）、NF（欄位數）。

## 相關文件

- [sed.md](./sed.md) — 串流編輯器
- [grep.md](./grep.md) — 模式匹配
- [lex.md](./lex.md) — 詞法分析器產生器
