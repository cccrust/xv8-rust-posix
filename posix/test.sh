cargo build --release
PATH="target/release:$PATH" sh tools/tests/test_sh_basic.sh
PATH="target/release:$PATH" sh tools/tests/test_tools_core.sh