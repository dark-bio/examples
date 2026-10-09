#!/bin/sh
# Check the examples the way CI does. Every documentation link resolves, every
# app builds, and every app runs against each fixture root that holds its data.
# Apps with optional grants also run with those grants declined. Language ports
# print the same manifest apart from source URLs, and the same report except
# for the hello apps.
#
# Usage: tools/check.sh [app ...]
set -eu

cd "$(dirname "$0")/.."

apps=${*:-$(ls apps)}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

fail() {
  printf 'FAIL %s\n' "$*"
  failures=$((failures + 1))
}

# Compare one inline manifest with the facts its declarations supply.
test_manifest_facts() {
  awk -f tools/manifest.awk > "$work/manifest-facts"
  printf '%s\n' "$2" > "$work/manifest-expected"
  diff "$work/manifest-expected" "$work/manifest-facts" || fail "manifest reader $1"
}

# Check table, dotted and inline declarations while descriptions stay opaque.
check_manifest_reader() {
  reader_failures=$failures

  # Read both grant lists and all three spellings of an input table name
  test_manifest_facts tables 'paths v1/genome/rsids/rs72921001
paths v1/genome/reference
optional v1/genome/rsids/rs671
input flavor
input toppings
input message' <<'TOML'
manifest = 1
[reads] # A comment can contain [inputs.fake]
paths = [
  'v1/genome/rsids/rs72921001', # A comment can contain ] and "quotes"
  "v1/genome/reference",
]
optional = ['v1/genome/rsids/rs671']
[inputs.flavor]
type = 'text'
prompt = 'Flavor'
max = 32
[ inputs . "toppings" ]
type = 'text'
prompt = 'Toppings'
max = 32
[inputs.'message']
type = 'text'
prompt = 'Message'
max = 32
TOML

  # Dotted declarations keep source order and emit each input name once
  test_manifest_facts dotted 'paths v1/genome/rsids/rs72921001
optional v1/genome/rsids/rs671
input z
input a' <<'TOML'
manifest = 1
reads.paths = ['v1/genome/rsids/rs72921001']
reads.optional = ["v1/genome/rsids/rs671"]
inputs."z".type = 'text'
inputs.'a'.type = 'text'
inputs.a.prompt = 'A'
inputs.z.prompt = 'Z'
inputs.a.max = 1
inputs.z.max = 1
TOML

  # Entries in the inputs table accept inline values and dotted fields
  test_manifest_facts input_entries 'input flavor
input toppings
input message' <<'TOML'
manifest = 1
[inputs]
"fl\u0061vor" = { type = 'text', prompt = 'Flavor', max = 32 }
'toppings'.type = 'text'
message.type = 'text'
message.prompt = 'Message'
toppings.prompt = 'Toppings'
message.max = 32
toppings.max = 32
TOML

  # Top-level inline tables accept comments, newlines and trailing commas
  test_manifest_facts inline 'paths v1/genome/reference
optional v1/genome/rsids/rs671
input z
input a' <<'TOML'
manifest = 1
reads = { paths = ["v1/genome/\x72eference"], optional = [
  'v1/genome/rsids/rs671',
] }
inputs = {
  'z' = {type = 'text', prompt = 'Z', max = 1}, # The first question
  "a" = {
    type = 'text',
    prompt = 'A',
    max = 1,
  },
}
TOML

  # Fenced declarations inside a literal description remain string content
  test_manifest_facts literal_description 'paths v1/genome/reference
input real' <<'TOML'
manifest = 1
[listing]
description = '''
```toml
[reads]
paths = ['v1/genome/rsids/rs1']
optional = ["v1/genome/rsids/rs2"]
[inputs.x]
type = 'text'
inputs.fake.type = 'text'
```
'''
[reads]
paths = ['v1/genome/reference']
[inputs.real]
type = 'text'
prompt = 'Real'
max = 1
TOML

  # Escaped quotes keep a basic description open across fenced declarations
  test_manifest_facts basic_description 'optional v1/genome/rsids/rs671
input real' <<'TOML'
manifest = 1
listing.description = """
The text contains \"\"\" and two quotes "".
```toml
[reads]
paths = ['v1/genome/rsids/rs1']
[inputs.x]
type = "text"
reads.optional = ['v1/genome/rsids/rs2']
```
"""
reads.optional = ['v1/genome/rsids/rs671']
inputs.real = {type = 'text', prompt = 'Real', max = 1}
TOML

  # Unsupported names and string values never become partial facts
  test_manifest_facts unsupported 'paths v1/genome/reference
input real' <<'TOML'
manifest = 1
reads.paths = ["v1/genome/rsids/rs\n1", 'v1/genome/reference']
"inputs.fake" = {type = 'text', prompt = 'Fake', max = 1}
[inputs."bad.name"]
type = 'text'
[inputs.real]
type = 'text'
prompt = 'Real'
max = 1
[listing]
inputs.fake = {type = 'text', prompt = 'Fake', max = 1}
TOML

  [ "$reader_failures" -ne "$failures" ] || echo "checked the manifest reader"
}

# A markdown link is `](target)`, and only the relative ones name a file here.
check_links() {
  for page in README.md fixtures/README.md docs/*.md apps/*/README.md; do
    for target in $(grep -o ']([^)]*)' "$page" | sed 's/^](//; s/)$//'); do
      case $target in
        http://*|https://*|'#'*) continue ;;
      esac
      [ -e "$(dirname "$page")/${target%%#*}" ] || fail "$page links to missing $target"
    done
  done
  echo "checked documentation links"
}

# The default tree, plus every root beside it holding a narrower case.
fixture_roots() {
  echo fixtures
  for root in fixtures/*/; do
    [ -d "$root/v1" ] && echo "${root%/}"
  done
}

# Run an app, accepting a stop when its required data is missing.
# An optional third argument of off declines optional grants in a separate run.
run_app() {
  # Store declined-grant runs separately so language comparisons keep defaults
  app=$1 root=$2 output=$work/output-$1-$(basename "$2")${3:+-optional-off}

  # Accept a successful report or the expected stop for a missing required grant
  if OPTIONAL="${3:-${OPTIONAL:-}}" sh tools/run.sh "${BUILD:-build}/$app.wasm" "$root" "apps/$app/inputs" > "$output" 2>&1; then
    sed -n '/^== report pass ==$/,$p' "$output" > "$work/report-$app-$(basename "$root")${3:+-optional-off}"
  elif [ "$?" -ne 3 ]; then
    fail "$app on $root${3:+ (OPTIONAL=$3)}"
    cat "$output"
  fi
}

# Build every selected app and run its grants on each fixture root.
build_and_run() {
  for app in $apps; do
    # Build once for all fixture roots and grant choices
    if ! sh tools/build.sh "$app" > "$work/build-$app" 2>&1; then
      fail "$app does not build"
      cat "$work/build-$app"
      continue
    fi

    # Capture each built module's manifest without its port-specific source URL
    if ! "${WASMTIME:-wasmtime}" "${BUILD:-build}/$app.wasm" > "$work/manifest-$app" 2> "$work/manifest-error-$app"; then
      fail "$app does not print its manifest"
      cat "$work/manifest-error-$app"
      continue
    fi
    sed '/^source = /d' "$work/manifest-$app" > "$work/manifest-compared-$app"
    awk -f tools/manifest.awk "$work/manifest-$app" > "$work/facts-$app"

    # Keep the default report for language comparisons and check declined grants
    for root in $(fixture_roots); do
      run_app "$app" "$root"
      if grep -q '^optional ' "$work/facts-$app"; then
        run_app "$app" "$root" off
      fi
    done
  done
  echo "built and ran $(echo "$apps" | wc -w | tr -d ' ') apps"
}

# Every port answers exactly like the Rust version it was ported from, so one
# cannot drift from the others. The hello apps differ on purpose, each greeting
# in its own language.
compare_languages() {
  for app in $apps; do
    family=$(printf '%s' "$app" | sed -E 's/-(c|go|python|rust)$//')
    reference=$family-rust
    case $family in
      01-hello) continue ;;
      "$app") continue ;;
    esac
    { [ "$app" != "$reference" ] && [ -d "apps/$reference" ]; } || continue
    for root in $(fixture_roots); do
      mine=$work/report-$app-$(basename "$root")
      theirs=$work/report-$reference-$(basename "$root")
      { [ -e "$mine" ] && [ -e "$theirs" ]; } || continue
      diff "$theirs" "$mine" > /dev/null || fail "$app differs from $reference on $root"
    done
  done
  echo "compared language versions"
}

# Compare each port's manifest with Rust when both were built in this run.
compare_manifests() {
  for app in $apps; do
    family=$(printf '%s' "$app" | sed -E 's/-(c|go|python|rust)$//')
    reference=$family-rust
    [ "$family" != "$app" ] || continue
    { [ "$app" != "$reference" ] && [ -d "apps/$reference" ]; } || continue
    mine=$work/manifest-compared-$app
    theirs=$work/manifest-compared-$reference
    { [ -e "$mine" ] && [ -e "$theirs" ]; } || continue
    diff "$theirs" "$mine" > /dev/null || fail "$app manifest differs from $reference"
  done
  echo "compared language manifests"
}

check_links
check_manifest_reader
build_and_run
compare_languages
compare_manifests

[ "$failures" -eq 0 ] || { printf '%d check(s) failed\n' "$failures"; exit 1; }
echo "all checks passed"
