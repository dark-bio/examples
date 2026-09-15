#!/bin/sh
# Run an Ark app the way the device does: read its manifest, mount only the
# datasets it declares (read-only), then run it with the data directory as its
# first argument. Invoked by `make run`. See docs/05-running.md.
#
# Usage: tools/run.sh <module.wasm> <fixtures-dir>
set -eu

wasm="$1"
fixtures="$2"
wasmtime="${WASMTIME:-wasmtime}"

# The manifest pass: run with no arguments, capture the TOML it prints.
manifest="$("$wasmtime" "$wasm")"
echo "== manifest pass =="
printf '%s\n' "$manifest"

# Mount only the datasets the manifest declares, each at its own path, so the app
# sees exactly what it asked for and nothing else, like the device does.
dirargs=""
for dataset in $(printf '%s\n' "$manifest" | grep -oE '"v1/[^"]*"' | tr -d '"'); do
  if [ ! -d "$fixtures/$dataset" ]; then
    printf 'fixture root has no declared dataset: %s\n' "$dataset" >&2
    exit 1
  fi
  dirargs="$dirargs --dir $fixtures/$dataset::/$dataset"
done

echo
echo "== run pass =="
# shellcheck disable=SC2086
"$wasmtime" run $dirargs "$wasm" /
