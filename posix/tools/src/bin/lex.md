# Lex — 詞法分析器產生器

`lex`（Lexical Analyzer Generator）從規則檔案產生 C 語言的詞法分析器。Lex 由 Mike Lesk 與 Eric Schmidt 在 AT&T Bell Labs 開發。規則包含正則表達式與對應的動作（C 程式碼），`lex` 編譯為確定性有限狀態自動機（DFA）。Lex 通常與 Yacc 一起使用，形成編譯器的前端。

## 相關文件

- [yacc.md](./yacc.md) — 語法分析器產生器
- [awk.md](./awk.md) — 模式掃描語言
