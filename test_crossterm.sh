# riscv64 編譯成功（所有工具 + crossterm）
cargo build --release -p tools --target riscv64gc-unknown-none-elf --features crossterm

# host 編譯 + 測試全部通過
cargo build -p tools --release
cd posix && bash test.sh
# Results: PASS: 33 (shell) + PASS: 21 (core tools)