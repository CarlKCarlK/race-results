cargo install wasm-pack

cargo test -- --nocapture
cargo run --example esr

cargo check --target wasm32-unknown-unknown --features alloc --no-default-features

# from tests/wasm-demo
wasm-pack build --target web

# test native
cargo test
# cmk how come this doesn't work? (update article?)
cargo test --features alloc --no-default-features
# check and test WASM
cargo check --target wasm32-unknown-unknown --features alloc --no-default-features
wasm-pack test --chrome --headless --features alloc --no-default-features


@REM doesn't work
@REM install wismer
cargo install wasmer-cli --features singlepass,cranelift
set path=C:\Users\carlk\.cargo\bin;%path% 
@REM wasmer run --entry-function main target/wasm32-unknown-unknown/debug/examples/esr.wasm
@REM wasmer run --entrypoint main --args "arg1 arg2" target/wasm32-unknown-unknown/debug/examples/esr.wasm
