set -x
./mkfs.sh
./test.sh
rm -f /tmp/testmode
cargo run --release