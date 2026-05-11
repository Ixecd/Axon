# Axon — Rust 开发 Makefile
# KubePivot 同谱: make dev = build + test + lint (无 race，Rust 编译时保证)

ROOT_DIR := $(shell pwd)
VERSION ?= v0.1.0
ARCH ?= amd64
REGISTRY_PREFIX ?= qingchun22
PROJECT_NAME := axon
DOCKERFILE := build/docker/$(PROJECT_NAME)/Dockerfile

.DEFAULT_GOAL := dev

# ================================================================
# Dev (default)
# ================================================================

.PHONY: dev
dev: build test lint
	@echo "✅ Axon dev 通过"

# ================================================================
# Build
# ================================================================

.PHONY: build
build:
	cargo build
	@echo "✅ cargo build"

.PHONY: build.release
build.release:
	cargo build --release
	@echo "✅ cargo build --release"

# ================================================================
# Test
# ================================================================

.PHONY: test
test:
	cargo test
	@echo "✅ cargo test"

.PHONY: test.lib
test.lib:
	cargo test --lib
	@echo "✅ cargo test --lib"

# ================================================================
# Lint
# ================================================================

.PHONY: lint
lint: fmt.check clippy

.PHONY: fmt
fmt:
	cargo fmt
	@echo "✅ cargo fmt"

.PHONY: fmt.check
fmt.check:
	cargo fmt --check
	@echo "✅ cargo fmt --check"

.PHONY: clippy
clippy:
	cargo clippy --all-targets -- -D warnings
	@echo "✅ cargo clippy"

.PHONY: clippy.fix
clippy.fix:
	cargo clippy --fix --all-targets -- -D warnings
	@echo "✅ cargo clippy --fix"

# ================================================================
# Bench
# ================================================================

.PHONY: bench
bench:
	cargo bench
	@echo "✅ cargo bench"

# ================================================================
# Fuzz (S+ 正确性 — 24h before release)
# ================================================================

.PHONY: fuzz.router
fuzz.router:
	cargo fuzz run router -- -max_total_time=3600
	@echo "✅ router fuzz 1h"

.PHONY: fuzz.mpc
fuzz.mpc:
	cargo fuzz run mpc_sig -- -max_total_time=3600
	@echo "✅ mpc sig fuzz 1h"

# ================================================================
# Audit (weekly)
# ================================================================

.PHONY: audit
audit:
	cargo audit
	cargo deny check
	@echo "✅ cargo audit + deny"

# ================================================================
# Docker
# ================================================================

include scripts/make-rules/deploy.mk

.PHONY: image
image: build.release
	docker build -t $(REGISTRY_PREFIX)/$(PROJECT_NAME)-$(ARCH):$(VERSION) -f $(DOCKERFILE) .

.PHONY: image.push
image.push:
	docker push $(REGISTRY_PREFIX)/$(PROJECT_NAME)-$(ARCH):$(VERSION)

# ================================================================
# Clean
# ================================================================

.PHONY: clean
clean:
	cargo clean
	@echo "✅ cargo clean"
