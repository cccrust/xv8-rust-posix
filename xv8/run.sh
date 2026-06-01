set -x
cargo build --release
./mkfs.sh
# ./setup_net.sh
cargo run --release
