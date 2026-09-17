#!/bin/sh
# Install the toolchains every job needs, at the versions the workflow pins.
set -eu

curl -fsSL "https://github.com/bytecodealliance/wasmtime/releases/download/v$WASMTIME_VERSION/wasmtime-v$WASMTIME_VERSION-x86_64-linux.tar.xz" | tar -xJ
sudo install "wasmtime-v$WASMTIME_VERSION-x86_64-linux/wasmtime" /usr/local/bin/

curl -fsSL "https://github.com/WebAssembly/binaryen/releases/download/version_$BINARYEN_VERSION/binaryen-version_$BINARYEN_VERSION-x86_64-linux.tar.gz" | tar -xz
sudo cp -r "binaryen-version_$BINARYEN_VERSION/bin" "binaryen-version_$BINARYEN_VERSION/lib" /usr/local/

curl -fsSL "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_SDK_VERSION%.*}/wasi-sdk-$WASI_SDK_VERSION-x86_64-linux.tar.gz" | tar -xz
sudo mv "wasi-sdk-$WASI_SDK_VERSION-x86_64-linux" /opt/wasi-sdk

wasmtime --version
wasm-opt --version
/opt/wasi-sdk/bin/clang --version | head -n 1
