# Yacc — 語法分析器產生器

`yacc`（Yet Another Compiler-Compiler）從 LALR(1) 語法規則產生解析器（parser）的 C 原始碼。由 Stephen C. Johnson 於 1975–1978 年在 Bell Labs 開發。語法檔案包含語法規則（BNF-like）與每個規則對應的動作（C 程式碼）。Yacc 生成的解析器使用查表驅動的 LR 解析。通常與 Lex 協同工作。

## 相關文件

- [lex.md](./lex.md) — 詞法分析器產生器
- [awk.md](./awk.md) — 模式掃描語言
- [c99.md](./c99.md) — C 編譯器
