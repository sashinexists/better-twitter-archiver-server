# build.sh
mkdir target/release
cp tweets.db target/release
cargo build --release