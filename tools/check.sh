#!/bin/sh
# Check the examples the way CI does: every documentation link resolves, every
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
  printf 'FAIL %s\n' "$1"
  failures=$((failures + 1))
}

# Every relative link in the documentation points at something that exists.
for page in README.md fixtures/README.md docs/*.md apps/*/README.md; do
  for target in $(sed -n 's/.*](\([^)#h][^)#]*\).*/\1/p' "$page"); do
    [ -e "$(dirname "$page")/$target" ] || fail "$page links to missing $target"
  done
done
echo "checked documentation links"

# The fixture roots are the default tree plus every root beside it.
roots=fixtures
for dir in fixtures/*/; do
  [ -d "$dir/v1" ] && roots="$roots ${dir%/}"
done

for app in $apps; do
  if ! make build APP="$app" > "$work/build.$app" 2>&1; then
    fail "$app does not build"
    cat "$work/build.$app"
    continue
  fi
  for root in $roots; do
    # An app whose data is absent from a root stops before its run pass, which
    # is the same thing an Ark does with a grant it cannot satisfy.
    if sh tools/run.sh "build/$app.wasm" "$root" > "$work/$app.$(basename "$root")" 2>&1; then
      sed -n '/^== run pass ==$/,$p' "$work/$app.$(basename "$root")" \
        > "$work/report.$app.$(basename "$root")"
    elif grep -q "no declared dataset" "$work/$app.$(basename "$root")"; then
      rm -f "$work/$app.$(basename "$root")"
    else
      fail "$app on $root"
      cat "$work/$app.$(basename "$root")"
    fi
  done
done
echo "built and ran $(echo "$apps" | wc -w | tr -d ' ') apps over $(echo "$roots" | wc -w | tr -d ' ') fixture roots"

# Language versions of one demo answer identically, so a port cannot drift.
# The hello apps are the exception, since each greets in its own language.
for report in "$work"/report.*; do
  [ -e "$report" ] || continue
  name=${report##*/report.}
  case $name in
    01-hello-*) continue ;;
    *-c.*|*-go.*|*-python.*) ;;
    *) continue ;;
  esac
  root=${name##*.}
  family=${name%.*}
  family=${family%-c}
  family=${family%-go}
  family=${family%-python}
  reference="$work/report.$family-rust.$root"
  [ -e "$reference" ] || reference="$work/report.$family.$root"
  [ -e "$reference" ] || continue
  diff "$reference" "$report" >/dev/null || fail "$name differs from $(basename "$reference")"
done
echo "compared language versions"

[ "$failures" -eq 0 ] || { printf '%d check(s) failed\n' "$failures"; exit 1; }
echo "all checks passed"
