cargo test -- --nocapture
cargo run --example esr


# install wismer
cargo install wasmer-cli --features singlepass,cranelift
set path=C:\Users\carlk\.cargo\bin;%path% 


wasmer run --entry-function main target/wasm32-unknown-unknown/debug/examples/esr.wasm

wasmer run --entrypoint main --args "arg1 arg2" target/wasm32-unknown-unknown/debug/examples/esr.wasm
