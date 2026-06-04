cargo check --manifest-path xv8rust/Cargo.toml --target riscv64gc-unknown-none-elf
cargo check --manifest-path xv8rust/xv8-axum-smoke/Cargo.toml
cargo run --manifest-path xv8rust/xv8-axum-smoke/Cargo.toml
