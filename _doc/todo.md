# xv8-rust-posix 開發規劃

從 v1.3 開始，文件統一放在 `_doc/` 下。各子專案原有的 `_doc/` 維持不動，新版本只寫在這裡。

---

## v1.3 — Shell 語法修正 ✅

**已於 `_doc/v1.3.md` 記錄。** 三個 bug 全部修復（bash.rs line 193/229/251, sh.rs line 68-70）。33/33 shell 測試通過。

---

## v1.4 — ls -R 與 diff LCS ✅

**已於 `_doc/v1.4.md` 記錄。** 檢查後發現兩者早已實作完成（ls.rs:143/244-253, diff.rs:139-177）。7 個測試全過。

---

## v1.5 — riscv64 Shell 相容 + tcgetattr/tcsetattr 驗證 ✅

**已於 `_doc/v1.5.md` 記錄。** sh.rs 現在可在 riscv64 上成功編譯（5 項修正）。214 cargo tests + 33 shell + 21 core tools 全過。

---

## v1.6 — riscv64 Crossterm（xv8-user-std 大幅擴充）

**目標：讓 crossterm 能在 riscv64gc-unknown-none-elf 上編譯，使 vi/vim 可在 xv8 內執行。**

| 項目 | 說明 |
|------|------|
| `fmt::Display` | xv8-user-std 需補完 Display trait 實作（目前 core 有但 std 層缺） |
| `ops` module | 補 `std::ops` re-export（Deref, Index, Range 等） |
| `Vec` re-export | 確保 `std::vec::Vec` 可正常使用 |
| 其他缺少的 std 項目 | 依 crossterm 編譯錯誤逐一補齊 |

**狀態：目前 `xv8-user-std` 只提供 POSIX 工具所需的最小 std 子集。crossterm 依賴更多 std 基礎設施（error trait 鏈、格式化、迭代器組合子等），需要大規模補強。**

**驗收：**
```bash
cargo build --release -p tools --target riscv64gc-unknown-none-elf --features crossterm
# vi, vim 都應成功編譯
```

---

## v1.7 — ex, fc 與選用工具 ✅

**已於 `_doc/v1.7.md` 記錄。** `ex` 和 `fc` 皆已實作，host 與 riscv64 編譯成功。

---

## v1.8 — iconv 改進 ✅

**已於 `_doc/v1.8.md` 記錄。**

---

## v2.0 — users/who 強化 + 小型選用工具 ✅

**已於 `_doc/v2.0.md` 記錄。** who 強化（支援 -a、-b、-H、-q、-r、-T、-u）、users 強化（libc 正規 API）、last/rev/col/look 新增。12 項測試全部通過。

---

## v2.1 — xv8 核心網路強化（TCP、HTTP、TLS）

**規劃中，見 `_doc/todo2.md`。**

---

## v2.2 — xv8rust/net bridge + 網路工具移植 ✅

**已於 `_doc/v2.2.md` 記錄。**
- Step 1 ✅：`xv8rust/xv8-net/` 最小 `std::net` 相容層（host + riscv64 雙目標編譯通過）
- Step 2 ✅：`net/libnet` platform abstraction + `net/tools` riscv64 build
- Step 3 ✅：xv8 QEMU 整合測試（12/12 passed），root `test.sh` 全專案合併測試（9/9 passed）

## v2.3 — HTTP 支援（xv8 QEMU 內 httpd + httpget）✅

**已於 `_doc/v2.3.md` 記錄。**
- `httpget` + `httpd` 雙工具，host + riscv64 雙目標編譯
- xv8 QEMU `_http` 測試通過（13/13）
- 不使用第三方 HTTP crate，純 `libnet::TcpStream` 實作

## v2.4 — xv8 QEMU Async Runtime（`block_on` + `Sleep`）✅

**已於 `_doc/v2.4.md` 記錄。**
- `xv8/user/testbin/async.rs` — 自包含的 async executor（zero extern deps）
- 完全避免 `csrr sstatus`（因 `riscv64gc-unknown-none-elf` 的 atomic 實作會使用 supervisor CSR）
- 使用 `Pin<&mut F>` on stack + dummy waker + poll-loop（zero atomics）
- `Sleep` future 由 `uptime` 驅動，單執行緒
- xv8 QEMU `_async` 測試通過（14/14）：6 項 subtests
- Root `test.sh` 全專案合併測試（9/9 passed）

## 版本對照

| 版本 | 主要內容 | 依賴 |
|------|---------|------|
| v1.3 | Shell 語法修正 | 無 |
| v1.4 | ls -R, diff LCS | 無 |
| v1.5 | riscv64 shell 相容 + tc* 驗證 | v1.3 (shell 先修好) |
| v1.6 | riscv64 crossterm (xv8-user-std 擴充) | v1.5 (riscv64 環境) |
| v1.7 | ex, fc | v1.3 (shell 依賴) |
| v1.8 | iconv 編碼擴充 | 無 |
| v2.0 | users/who 強化 + rev/col/look/last | 無 |
| v2.1 | xv8 核心網路（TCP、HTTP、TLS） | v2.0 |
| v2.2 | xv8rust/net bridge + 網路工具移植 ✅ | v2.1 |
| v2.3 | HTTP 支援（httpd + httpget）✅ | v2.2 |
| v2.4 | Async Runtime（`block_on` + `Sleep`）✅ | v2.3 |
| v2.5 | Kernel epoll + async I/O + tokio compat ✅ | v2.4 |
| v2.6 | POSIX shell 整合 xv8 + fix argv 傳遞 ✅ | v2.5 |

### Next
- v2.7: Shell bugs fix (`elif`, empty-string args, `test = ""`, single-quote expansion)
