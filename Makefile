VERSION ?= $(shell git describe --tags --dirty 2>/dev/null | sed 's/^v//' || echo "0.0.1")
COMMIT  ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
DATE    ?= $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
BINARY  := bb

.PHONY: build install test fmt fmt-check lint hooks-install clean help

build: ## Build binary
	@BB_BUILD_COMMIT=$(COMMIT) BB_BUILD_DATE=$(DATE) cargo build --manifest-path rust/Cargo.toml -p bb-cli --bin $(BINARY)

install: ## Install release binary to ~/.local/bin
	@BB_BUILD_COMMIT=$(COMMIT) BB_BUILD_DATE=$(DATE) cargo build --manifest-path rust/Cargo.toml -p bb-cli --bin $(BINARY) --release
	@mkdir -p $(HOME)/.local/bin
	@cp rust/target/release/$(BINARY) $(HOME)/.local/bin/$(BINARY)

test: ## Run all tests
	@cargo test --manifest-path rust/Cargo.toml

fmt: ## Format Rust source files
	@cargo fmt --manifest-path rust/Cargo.toml --all

fmt-check: ## Check Rust formatting without modifying files
	@cargo fmt --manifest-path rust/Cargo.toml --all --check

lint: ## Run clippy with warnings denied
	@cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings

hooks-install: ## Configure git to use repo-managed hooks in .githooks
	@git config core.hooksPath .githooks

clean: ## Remove Rust build artifacts
	@rm -rf rust/target

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":[^:]*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
