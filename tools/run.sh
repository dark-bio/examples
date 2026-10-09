#!/bin/sh
# Run an Ark app with its required datasets and available optional datasets
# mounted read-only. OPTIONAL=off leaves every optional dataset unmounted.
# Declared inputs come from answer files, copied read-only without validation.
# Start the report pass with the data directory as its first argument.
# Invoked by `make run`. See docs/05-running.md.
#
# Usage: tools/run.sh <module.wasm> <fixtures-dir> [inputs-dir]
set -eu

wasm="$1"
fixtures="$2"
inputs="${3:-}"
wasmtime="${WASMTIME:-wasmtime}"

# Capture the TOML from the manifest pass, which runs with no arguments
manifest="$("$wasmtime" "$wasm")"
echo "== manifest pass =="
printf '%s\n' "$manifest"

# wasmtime lets a guest write to every directory it mounts, so grants and
# inputs are mounted from copies in a scratch tree without write permission
stage="$(mktemp -d)"
trap 'chmod -R u+w "$stage"; rm -rf "$stage"' EXIT

# Read required and optional arrays only inside the reads table
printf '%s\n' "$manifest" | awk '
  /^[[:space:]]*\[/ {
    reads = $0 ~ /^[[:space:]]*\[[[:space:]]*reads[[:space:]]*\][[:space:]]*(#.*)?$/
    grant = ""
    next
  }
  !reads { next }
  {
    # Keep the grant kind across lines until its array closes
    line = $0
    if (match(line, /^[[:space:]]*(paths|optional)[[:space:]]*=[[:space:]]*\[/)) {
      grant = line
      sub(/^[[:space:]]*/, "", grant)
      sub(/[[:space:]]*=.*/, "", grant)
      line = substr(line, RLENGTH + 1)
    }
    if (grant == "") next

    # Read either string form, stopping at comments and the closing bracket
    while (match(line, /"[^"]*"|\047[^\047]*\047|#|\]/)) {
      token = substr(line, RSTART, RLENGTH)
      if (token == "#") break
      if (token == "]") {
        grant = ""
        break
      }
      print grant, substr(token, 2, length(token) - 2)
      line = substr(line, RSTART + RLENGTH)
    }
  }
' > "$stage/grants"

# Mount each accepted grant at its own path from a copy without write permission
set --
while read -r grant dataset; do
  if [ "$grant" = optional ]; then
    if [ "${OPTIONAL:-}" = off ]; then
      printf 'optional dataset left unmounted, OPTIONAL=off: %s\n' "$dataset" >&2
      continue
    fi
    if [ ! -d "$fixtures/$dataset" ]; then
      printf 'optional dataset left unmounted, not in this fixture root: %s\n' "$dataset" >&2
      continue
    fi
  fi
  if [ ! -d "$fixtures/$dataset" ]; then
    printf 'fixture root has no declared dataset: %s\n' "$dataset" >&2
    exit 1
  fi
  mkdir -p "$stage/$(dirname "$dataset")"
  cp -R "$fixtures/$dataset" "$stage/$dataset"
  set -- "$@" --dir "$stage/$dataset::/$dataset"
done < "$stage/grants"

# Read the input table headers in the form the examples print
printf '%s\n' "$manifest" | awk '
  /^[[:space:]]*\[inputs\.[a-z][a-z0-9_-]*\][[:space:]]*(#.*)?$/ {
    name = $0
    sub(/^[[:space:]]*\[inputs\./, "", name)
    sub(/\].*$/, "", name)
    print name
  }
' > "$stage/input-names"

# Copy only declared answers and mount them together under the data root
if [ -s "$stage/input-names" ]; then
  if [ ! -d "$inputs" ]; then
    printf 'declared inputs require an answers directory as the third argument: %s\n' "${inputs:-none provided}" >&2
    exit 1
  fi
  mkdir -p "$stage/inputs"
  while read -r name; do
    if [ ! -f "$inputs/$name" ]; then
      printf 'answers directory has no file for declared input: %s\n' "$name" >&2
      exit 1
    fi
    cp "$inputs/$name" "$stage/inputs/$name"
  done < "$stage/input-names"
  set -- "$@" --dir "$stage/inputs::/inputs"
fi
chmod -R a-w "$stage"

# Start the report pass with the accepted grants, inputs and the data root
echo
echo "== report pass =="
"$wasmtime" run "$@" "$wasm" /
