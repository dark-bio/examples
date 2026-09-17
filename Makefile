# Build and run the Ark example apps.
#
# Usage:
#   make build                      build every app
#   make build APP=01-hello-rust    build one app
#   make run                        build and run every app
#   make run   APP=01-hello-rust    build and run one app
#   make clean                      remove build outputs
#
# Every app builds to a single module at build/<app>.wasm, whatever its
# language. tools/build.sh picks the toolchain from the files an app has, and
# tools/run.sh runs the module against the fixtures/ tree, which stands in for
# the data an Ark would mount. See docs/05-running.md.

FIXTURES ?= fixtures
BUILD ?= build
APPS := $(sort $(notdir $(wildcard apps/*)))
SELECTED := $(or $(strip $(APP)),$(APPS))

# The scripts read these, and any toolchain override, from the environment.
export BUILD WASMTIME WASI_SDK PYTHON TINYGO WASM_OPT

.PHONY: all build run clean list FORCE

all: build

list:
	@echo "Apps:"; for a in $(APPS); do echo "  $$a"; done
	@echo "Build all: make build   Run all: make run   One: make run APP=<name>"

build: $(addprefix build-,$(SELECTED))

run: $(addprefix run-,$(SELECTED))

build-%: FORCE
	@sh tools/build.sh "$*"

# A header keeps the apps apart when every one of them runs in turn.
run-%: FORCE
	@if [ -z "$(strip $(APP))" ]; then echo; echo "===== $* ====="; fi
	@sh tools/build.sh "$*"
	@sh tools/run.sh "$(BUILD)/$*.wasm" "$(FIXTURES)"

clean:
	@rm -rf $(BUILD) apps/*/target apps/*/*.wasm
	@echo "cleaned build outputs"
