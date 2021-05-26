# #!/bin/bash
# set -eu

# NAME="app"

# rustup target list | grep installed | grep -q wasm || rustup target add wasm32-unknown-unknown

# time cargo build --release --target wasm32-unknown-unknown
# cp target/wasm32-unknown-unknown/release/$NAME.wasm ./

# # https://github.com/WebAssembly/binaryen
# wasm-opt -Oz -o $NAME.wasm1 $NAME.wasm

# mv $NAME.wasm1 $NAME.wasm

# # wasm strip made it ... bigger? wtf
# # https://github.com/WebAssembly/wabt
# #wasm-strip $NAME.wasm

# du -sh $NAME.wasm
