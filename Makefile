# Build and run the Ark example apps.
#
# Usage:
#   make build                      build every app
#   make build APP=01-hello-rust    build one app
#   make run                        build and run every app
#   make run   APP=01-hello-rust    build and run one app
#   make clean                      remove build outputs
#
# Every app builds to a single module at build/<app>.wasm, whatever its language.
# The run target uses the wasmtime CLI against the fixtures/ tree, which stands
# in for the data an Ark would mount. See docs/05-running.md.

FIXTURES := fixtures
WASMTIME := wasmtime
BUILD := build
PYTHON := python3
TINYGO := tinygo
WASM_OPT := wasm-opt
APPS := $(sort $(notdir $(wildcard apps/*)))
BREW_LLVM := $(shell command -v brew >/dev/null 2>&1 && brew --prefix llvm 2>/dev/null)

ifeq ($(origin WASI_SDK), undefined)
ifneq ($(wildcard /opt/wasi-sdk/bin/clang),)
WASI_SDK := /opt/wasi-sdk
else ifneq ($(BREW_LLVM),)
WASI_SDK := $(BREW_LLVM)
else
WASI_SDK := /opt/wasi-sdk
endif
endif

WASM_FEATURES := --mvp-features --enable-mutable-globals --enable-sign-ext \
  --enable-nontrapping-float-to-int --enable-bulk-memory --enable-bulk-memory-opt \
  --enable-simd --enable-relaxed-simd --enable-multivalue --enable-reference-types \
  --enable-tail-call --enable-extended-const

ifeq ($(strip $(APP)),)
SELECTED_APPS := $(APPS)
else
SELECTED_APPS := $(strip $(APP))
endif

UNKNOWN_APPS := $(filter-out $(APPS),$(SELECTED_APPS))
GOALS_REQUIRING_APPS := $(filter all build run,$(or $(MAKECMDGOALS),all))

ifneq ($(GOALS_REQUIRING_APPS),)
ifneq ($(UNKNOWN_APPS),)
$(error unknown APP '$(UNKNOWN_APPS)'; valid apps: $(APPS))
endif
endif

.PHONY: all build run clean list FORCE

list:
	@echo "Apps:"; for a in $(APPS); do echo "  $$a"; done
	@echo "Build all: make build   Run all: make run   One: make run APP=<name>"

all: build

# Pick the language from Cargo.toml, go.mod, main.c or main.py.
# Every app produces one module in the build directory.
build: $(addprefix build-,$(SELECTED_APPS))

define build_app
set -e; \
dir="apps/$(1)"; \
out="$(BUILD)/$(1).wasm"; \
if [ ! -d "$$dir" ]; then \
  echo "unknown app: $(1)"; \
  echo "valid apps: $(APPS)"; \
  exit 1; \
fi; \
mkdir -p "$(BUILD)"; \
if ! command -v "$(WASM_OPT)" >/dev/null 2>&1; then echo "missing wasm-opt; install Binaryen or set WASM_OPT" >&2; exit 1; fi; \
if [ -f "$$dir/Cargo.toml" ]; then \
  ( cd "$$dir" && CARGO_ENCODED_RUSTFLAGS="$$(printf '%s\037%s\037%s' \
      '--remap-path-prefix=$(CURDIR)=.' \
      "--remap-path-prefix=$${CARGO_HOME:-$$HOME/.cargo}=cargo" \
      "--remap-path-prefix=$$(rustc --print sysroot)=rust")" \
    cargo build --locked --release --target wasm32-wasip1 ); \
  wasm=; for candidate in "$$dir"/target/wasm32-wasip1/release/*.wasm; do [ -e "$$candidate" ] || break; wasm=$$candidate; break; done; \
  if [ -z "$$wasm" ]; then echo "no wasm output found for $$dir"; exit 1; fi; \
  cp "$$wasm" "$$out"; \
  post="-O3"; \
elif [ -f "$$dir/go.mod" ]; then \
  if ! command -v "$(TINYGO)" >/dev/null 2>&1; then echo "missing TinyGo; install it or set TINYGO" >&2; exit 1; fi; \
  ( cd "$$dir" && "$(TINYGO)" build -target=wasip1 -opt=z -no-debug \
    -gc=precise -scheduler=asyncify -panic=trap -o app.wasm . ); \
  mv "$$dir/app.wasm" "$$out"; \
  post="-Os --converge"; \
elif [ -f "$$dir/main.c" ]; then \
  if [ ! -x "$(WASI_SDK)/bin/clang" ]; then echo "missing WASI clang; set WASI_SDK to its toolchain directory" >&2; exit 1; fi; \
  ( cd "$$dir" && "$(WASI_SDK)/bin/clang" --target=wasm32-wasip1 -Oz \
    -ffunction-sections -fdata-sections -Wl,--gc-sections -Wl,--strip-all main.c -o app.wasm ); \
  mv "$$dir/app.wasm" "$$out"; \
  post="-O4"; \
elif [ -f "$$dir/main.py" ]; then \
  if ! command -v "$(PYTHON)" >/dev/null 2>&1; then echo "missing CPython 3.13 or newer; set PYTHON to its executable" >&2; exit 1; fi; \
  "$(PYTHON)" tools/python_build.py "$$dir/main.py" "$$out" --sdk "$(WASI_SDK)"; \
  post="-Os --converge"; \
else \
  echo "don't know how to build $$dir"; \
  exit 1; \
fi; \
"$(WASM_OPT)" $(WASM_FEATURES) $$post "$$out" -o "$$out.tmp"; \
mv "$$out.tmp" "$$out"; \
echo "built $$out"
endef

build-%: FORCE
	@$(call build_app,$*)

# Run one app, or every app in turn when APP is unset.
run: $(addprefix run-,$(SELECTED_APPS))

run-%: FORCE
	@if [ -z "$(strip $(APP))" ]; then echo; echo "===== $* ====="; fi
	@$(call build_app,$*)
	@WASMTIME="$(WASMTIME)" sh tools/run.sh "$(BUILD)/$*.wasm" "$(FIXTURES)"

clean:
	@rm -rf $(BUILD) apps/*/target apps/*/*.wasm
	@echo "cleaned build outputs"
