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

# The manifest pass runs with no arguments, so capture the TOML it prints.
manifest="$("$wasmtime" "$wasm")"
echo "== manifest pass =="
printf '%s\n' "$manifest"

# Mount only the datasets the manifest declares, each at its own path, so the app
# sees exactly what it asked for and nothing else, like the device does.
# wasmtime lets a guest write to every directory it mounts, so the datasets are
# copied into a scratch tree without write permission and mounted from there.
stage="$(mktemp -d)"
trap 'chmod -R u+w "$stage"; rm -rf "$stage"' EXIT
set --
for dataset in $(printf '%s\n' "$manifest" | grep -oE '"v1/[^"]*"' | tr -d '"'); do
  if [ ! -d "$fixtures/$dataset" ]; then
    printf 'fixture root has no declared dataset: %s\n' "$dataset" >&2
    exit 1
  fi
  mkdir -p "$stage/$(dirname "$dataset")"
  cp -R "$fixtures/$dataset" "$stage/$dataset"
  set -- "$@" --dir "$stage/$dataset::/$dataset"
done
chmod -R a-w "$stage"

echo
echo "== run pass =="
"$wasmtime" run "$@" "$wasm" /
