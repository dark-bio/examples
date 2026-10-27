#!/bin/sh
# Run an app and its dependencies against local fixtures, with labeled streams.
# Modules are found by SHA-256 among the given files, or BUILD/*.wasm by default.
# Grants, inputs and completed outputs are read-only. Each app writes
# its declared outputs in a temporary directory, removed when the task ends.
# OPTIONAL=off leaves every optional dataset unmounted. INPUTS supplies answers
# when the third argument is empty, including untyped inputs for stage debugging.
# REPORT=<file> saves the main app's report exactly as it printed it.
# STDERR=<file> saves the main app's standard error exactly as it printed it.
# It checks only what it needs to run. The Ark checks every manifest and value
# rule, and the emulator runs the real firmware.
# The Ark's merged count, shared runs, limits and determinism are not reproduced.
# A missing required dataset exits with status 3; app failures exit with 1.
# Ctrl-C and SIGTERM stop the child and clean up, exiting with 130 and 143.
# A POSIX shell runs the passes and mounts; AWK parses and orders the task.
# See docs/05-running.md.
#
# Usage: tools/run.sh <module.wasm> <fixtures-dir> [inputs-dir] [module.wasm ...]
set -eu

# Report one local failure after the app's streams have been shown.
fail() {
  printf 'error: %s\n' "$2" >&2
  exit "$1"
}

# Stop and reap the runtime before restoring and removing its mount trees.
cleanup() {
  trap '' INT TERM
  if [ -n "$child" ]; then
    kill -KILL "$child" 2>/dev/null || :
    wait "$child" 2>/dev/null || :
  fi
  find "$stage" -type d -exec chmod u+rwx {} +
  rm -rf "$stage"
}

# Freeze files and directories after their contents have been prepared.
readonly_tree() {
  find "$1" -type f -exec chmod 444 {} +
  find "$1" -type d -exec chmod 555 {} +
}

# Run one pass, retain its exact streams and label them even when it fails.
run_pass() {
  pass_dir=$1 pass_phase=$2 pass_module=$3 pass_path=$4
  set -- run
  if [ "$pass_phase" = processing ]; then
    while IFS= read -r mount; do set -- "$@" --dir "$mount"; done < "$pass_dir/mounts"
  fi
  set -- "$@" "$stage/modules/$pass_module/module"
  [ "$pass_phase" != processing ] || set -- "$@" /

  # Waiting on an asynchronous child lets INT and TERM interrupt the shell
  "${WASMTIME:-wasmtime}" "$@" </dev/null > "$pass_dir/$pass_phase" 2> "$pass_dir/stderr" &
  child=$!
  pass_status=0
  wait "$child" || pass_status=$?
  child=

  # Save the main app's streams before handling a failed pass
  if [ "$pass_phase" = processing ] && [ "$pass_path" = . ]; then
    [ -z "${REPORT:-}" ] || cp "$pass_dir/$pass_phase" "$REPORT"
    [ -z "${STDERR:-}" ] || cp "$pass_dir/stderr" "$STDERR"
  fi

  # cat carries raw stream bytes without shell substitution or decoding
  printf '== %s pass: %s: stdout ==\n' "$pass_phase" "$pass_path"
  cat "$pass_dir/$pass_phase"
  printf '\n'
  printf '== %s pass: %s: stderr ==\n' "$pass_phase" "$pass_path" >&2
  cat "$pass_dir/stderr" >&2
  printf '\n' >&2
  [ "$pass_status" -eq 0 ] || fail 1 "$pass_path: $pass_phase pass failed (status $pass_status)"
}

# Reject incomplete invocations before allocating a scratch directory
[ "$#" -ge 2 ] || fail 2 'usage: tools/run.sh <module.wasm> <fixtures-dir> [inputs-dir] [module.wasm ...]'
module=$1 fixtures=$2 answers=${3:-${INPUTS:-}}
shift 2
[ "$#" -eq 0 ] || shift
if [ "$#" -eq 0 ]; then
  set -- "${BUILD:-build}"/*.wasm
  [ -e "$1" ] || set --
fi
tools=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
LC_ALL=C
export LC_ALL

stage=$(mktemp -d)
child=
trap cleanup 0
trap 'exit 130' INT
trap 'exit 143' TERM

# Hash private copies so both passes use exactly the bytes that resolved the pin
mkdir "$stage/modules"
sequence=0
for candidate in "$module" "$@"; do
  sequence=$((sequence + 1))
  mkdir "$stage/modules/$sequence"
  cp "$candidate" "$stage/modules/$sequence/module"
  if command -v sha256sum >/dev/null 2>&1; then
    digest=$(sha256sum < "$stage/modules/$sequence/module")
  else
    digest=$(shasum -a 256 < "$stage/modules/$sequence/module")
  fi
  printf '%s %s\n' "${digest%% *}" "$sequence" >> "$stage/modules/index"
done

# Collect each reachable module's manifest before computing the complete plan
tab=$(printf '\t')
printf '1\t.\n' > "$stage/queue"
touch "$stage/modules/1/queued"
while IFS="$tab" read -r id path; do
  run_pass "$stage/modules/$id" manifest "$id" "$path"
  awk -v task=pins -v task_path="$path" -f "$tools/manifest.awk" -f "$tools/task.awk" \
    "$stage/modules/$id/manifest" > "$stage/pins"
  while IFS="$tab" read -r name digest; do
    if [ "$path" = . ]; then pin_path=$name; else pin_path=$path/$name; fi
    pin=$(awk -v digest="$digest" '$1 == digest { print $2; exit }' "$stage/modules/index")
    [ -n "$pin" ] || fail 1 "$pin_path: no module found for sha256 $digest"
    if [ ! -e "$stage/modules/$pin/queued" ]; then
      touch "$stage/modules/$pin/queued"
      printf '%s\t%s\n' "$pin" "$pin_path" >> "$stage/queue"
    fi
  done < "$stage/pins"
done < "$stage/queue"
RUNNER_STAGE=$stage RUNNER_FIXTURES=$fixtures \
  awk -v task=plan -f "$tools/manifest.awk" -f "$tools/task.awk" \
  </dev/null > "$stage/plan"

# Carry out the plan only after all references and the order have been resolved
while IFS="$tab" read -r command app a b c d; do
  directory=$stage/app-$app
  case $command in
    app)
      mkdir "$directory"
      : > "$directory/mounts"
      ;;
    answer)
      [ -n "$answers" ] && [ -f "$answers/$a" ] ||
        fail 1 ".: answers directory has no file for declared input: $a"
      mkdir -p "$directory/inputs"
      cp "$answers/$a" "$directory/inputs/$a"
      ;;
    directory)
      mkdir -p "$directory/$a"
      ;;
    grant)
      grant=$b
      if [ ! -d "$fixtures/$grant" ] && [ ! -d "$directory/$grant" ]; then
        if [ "$a" = optional ]; then
          printf '%s: optional dataset not in this fixture root: %s\n' "$c" "$grant" >&2
          continue
        fi
        fail 3 "$c: fixture root has no declared dataset: $grant"
      fi
      mkdir -p "$directory/$grant"
      [ ! -d "$fixtures/$grant" ] || cp -RL "$fixtures/$grant/." "$directory/$grant/"
      printf '%s::/%s\n' "$directory/$grant" "$grant" >> "$directory/mounts"
      ;;
    freeze-grants)
      [ ! -d "$directory/v1" ] || readonly_tree "$directory/v1"
      ;;
    literal)
      mkdir -p "$directory/inputs"
      printf '%b' "$b" > "$directory/inputs/$a"
      ;;
    copy)
      mkdir -p "$directory/inputs"
      cp "$stage/app-$b/$c/$d" "$directory/inputs/$a"
      ;;
    choices)
      printf '%b' "$b" > "$stage/choices"
      awk 'FILENAME == ARGV[1] { picked[$0] = 1; next }
           $0 in picked { printf "%s%s", separator, $0; separator = "\n" }' \
        "$directory/inputs/$a" "$stage/choices" > "$stage/ordered"
      mv "$stage/ordered" "$directory/inputs/$a"
      ;;
    freeze-inputs)
      readonly_tree "$directory/inputs"
      printf '%s::/inputs\n' "$directory/inputs" >> "$directory/mounts"
      ;;
    dependency)
      printf '%s::/apps/%s\n' "$stage/app-$a/outputs" "$b" >> "$directory/mounts"
      ;;
    output)
      mkdir -p "$directory/outputs"
      : > "$directory/outputs/$a"
      chmod 600 "$directory/outputs/$a"
      ;;
    processing)
      if [ -d "$directory/outputs" ]; then
        chmod 555 "$directory/outputs"
        printf '%s::/outputs\n' "$directory/outputs" >> "$directory/mounts"
      fi
      run_pass "$directory" processing "$a" "$b"
      [ ! -d "$directory/outputs" ] || readonly_tree "$directory/outputs"
      ;;
    *) fail 1 'invalid planner command' ;;
  esac
done < "$stage/plan"
