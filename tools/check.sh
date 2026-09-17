#!/bin/sh
# Check the examples the way CI does. Every documentation link resolves, every
# app builds, every app runs against each fixture root that holds its data, and
# the language versions of one demo all print the same report.
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

# An app whose data a root lacks stops before its run pass, the way an Ark
# refuses a grant it cannot satisfy. Anything else is a failure.
run_app() {
  app=$1 root=$2 output=$work/output-$1-$(basename "$2")
  if sh tools/run.sh "${BUILD:-build}/$app.wasm" "$root" > "$output" 2>&1; then
    sed -n '/^== run pass ==$/,$p' "$output" > "$work/report-$app-$(basename "$root")"
  elif ! grep -q "no declared dataset" "$output"; then
    fail "$app on $root"
    cat "$output"
  fi
}

build_and_run() {
  for app in $apps; do
    if ! sh tools/build.sh "$app" > "$work/build-$app" 2>&1; then
      fail "$app does not build"
      cat "$work/build-$app"
      continue
    fi
    for root in $(fixture_roots); do
      run_app "$app" "$root"
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

check_links
build_and_run
compare_languages

[ "$failures" -eq 0 ] || { printf '%d check(s) failed\n' "$failures"; exit 1; }
echo "all checks passed"
