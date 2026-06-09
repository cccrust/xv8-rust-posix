# Attribute 列舉

定義 ANSI SGR 參數對應的文字屬性列舉，每個變體對應一或多個 SGR 碼：
`Bold`(1)、`Dim`(2)、`Italic`(3)、`Underlined`(4)、`SlowBlink`(5)、
`RapidBlink`(6)、`Reverse`(7)、`Hidden`(8)、`CrossedOut`(9) 等。
支援屬性的反轉運算（`ResetBold` 對應 SGR 22）以及完整重置（`Reset`，SGR 0）。
提供 `Display` 實作輸出對應的 ANSI 參數字串。
