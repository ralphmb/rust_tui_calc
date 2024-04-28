#/bin/zsh

here="$(dirname "$(readlink -f "$0")")"

dest="/usr/local/bin/calc"
cd $here
cargo build --release

cp ./target/release/rust_calc $dest
