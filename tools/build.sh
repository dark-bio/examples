#!/bin/sh
# Build one app into build/<app>.wasm, whatever language it is written in. The
# language comes from the files present, and every build is tuned for a small
# module, which uploads and starts faster on an Ark. Invoked by `make build`.
# See docs/05-running.md.
#
# Usage: tools/build.sh <app>
#
# WASI_SDK, PYTHON, TINYGO, WASM_OPT and BUILD override the toolchains and the
# output directory.
set -eu

cd "$(dirname "$0")/.."
root=$(pwd)

app=$1
dir=apps/$app
out=${BUILD:-build}/$app.wasm

if [ ! -d "$dir" ]; then
  echo "unknown app: $app" >&2
  echo "valid apps: $(ls apps | tr '\n' ' ')" >&2
  exit 1
fi

python=${PYTHON:-python3}
tinygo=${TINYGO:-tinygo}
wasm_opt=${WASM_OPT:-wasm-opt}

# A wasi-sdk install carries the sysroot the Python build needs. Homebrew's LLVM
# stands in for C alone, and WASI_SDK points at either one somewhere else.
if [ -z "${WASI_SDK:-}" ]; then
  WASI_SDK=/opt/wasi-sdk
  if [ ! -x "$WASI_SDK/bin/clang" ] && command -v brew >/dev/null 2>&1; then
    WASI_SDK=$(brew --prefix llvm 2>/dev/null || echo /opt/wasi-sdk)
  fi
fi

missing() {
  echo "$1" >&2
  exit 1
}

command -v "$wasm_opt" >/dev/null 2>&1 ||
  missing "missing wasm-opt; install Binaryen or set WASM_OPT"
mkdir -p "$(dirname "$out")"

if [ -f "$dir/Cargo.toml" ]; then
  # The size settings live in each app's Cargo.toml. These only keep the local
  # paths of this machine out of the module.
  ( cd "$dir" && CARGO_ENCODED_RUSTFLAGS="$(printf '%s\037%s\037%s' \
      "--remap-path-prefix=$root=." \
      "--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=cargo" \
      "--remap-path-prefix=$(rustc --print sysroot)=rust")" \
    cargo build --locked --release --target wasm32-wasip1 )
  wasm=
  for candidate in "$dir"/target/wasm32-wasip1/release/*.wasm; do
    [ -e "$candidate" ] || break
    wasm=$candidate
    break
  done
  [ -n "$wasm" ] || missing "no wasm output found for $dir"
  cp "$wasm" "$out"
  post="-O3"
elif [ -f "$dir/go.mod" ]; then
  command -v "$tinygo" >/dev/null 2>&1 ||
    missing "missing TinyGo; install it or set TINYGO"
  ( cd "$dir" && "$tinygo" build -target=wasip1 -opt=z -no-debug \
    -gc=precise -scheduler=asyncify -panic=trap -o app.wasm . )
  mv "$dir/app.wasm" "$out"
  post="-Os --converge"
elif [ -f "$dir/main.c" ]; then
  [ -x "$WASI_SDK/bin/clang" ] ||
    missing "missing WASI clang; set WASI_SDK to its toolchain directory"
  ( cd "$dir" && "$WASI_SDK/bin/clang" --target=wasm32-wasip1 -Oz \
    -ffunction-sections -fdata-sections -Wl,--gc-sections -Wl,--strip-all \
    main.c -o app.wasm )
  mv "$dir/app.wasm" "$out"
  post="-O4"
elif [ -f "$dir/main.py" ]; then
  command -v "$python" >/dev/null 2>&1 ||
    missing "missing CPython 3.13 or newer; set PYTHON to its executable"
  "$python" tools/python_build.py "$dir/main.py" "$out" --sdk "$WASI_SDK"
  post="-Os --converge"
else
  missing "don't know how to build $dir"
fi

# wasm-opt may only use the WebAssembly features an Ark's runtime accepts.
# shellcheck disable=SC2086
"$wasm_opt" --mvp-features --enable-mutable-globals --enable-sign-ext \
  --enable-nontrapping-float-to-int --enable-bulk-memory --enable-bulk-memory-opt \
  --enable-simd --enable-relaxed-simd --enable-multivalue --enable-reference-types \
  --enable-tail-call --enable-extended-const \
  $post "$out" -o "$out.tmp"
mv "$out.tmp" "$out"
echo "built $out"
