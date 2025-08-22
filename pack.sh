set -ex

cargo test
cargo build --release

mkdir machining-transpiler
cp -r .env target/release/machining-transpiler.exe templates/ machining-transpiler/

7z a machining-transpiler.zip machining-transpiler
rm -rf machining-transpiler