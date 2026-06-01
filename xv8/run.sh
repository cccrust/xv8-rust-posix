set -x
./mkfs.sh
./test.sh
./mkfs.sh
rm -f /tmp/testmode
cargo run --release
