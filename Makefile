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
# in for the data an Ark would mount. See docs/05-running-locally.md.

FIXTURES := fixtures
WASMTIME := wasmtime
BUILD := build
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

# Build one app to build/<app>.wasm, or every app when APP is unset. The
# toolchain is picked from the files present (Cargo.toml, go.mod, or main.c) and
# the resulting wasm is collected into the single build/ directory.
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
if [ -f "$$dir/Cargo.toml" ]; then \
  ( cd "$$dir" && cargo build --locked --release --target wasm32-wasip1 ); \
  wasm=; for candidate in "$$dir"/target/wasm32-wasip1/release/*.wasm; do [ -e "$$candidate" ] || break; wasm=$$candidate; break; done; \
  if [ -z "$$wasm" ]; then echo "no wasm output found for $$dir"; exit 1; fi; \
  cp "$$wasm" "$$out"; \
elif [ -f "$$dir/go.mod" ]; then \
  ( cd "$$dir" && GOOS=wasip1 GOARCH=wasm go build -o app.wasm . ); \
  mv "$$dir/app.wasm" "$$out"; \
elif [ -f "$$dir/main.c" ]; then \
  if [ ! -x "$(WASI_SDK)/bin/clang" ]; then echo "missing WASI clang; set WASI_SDK to its toolchain directory" >&2; exit 1; fi; \
  ( cd "$$dir" && "$(WASI_SDK)/bin/clang" --target=wasm32-wasip1 -O3 main.c -o app.wasm ); \
  mv "$$dir/app.wasm" "$$out"; \
else \
  echo "don't know how to build $$dir"; \
  exit 1; \
fi; \
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
