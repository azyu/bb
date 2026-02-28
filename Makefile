PKG     := bitbucket-cli/internal/version
VERSION ?= $(shell git describe --tags --dirty 2>/dev/null || echo "0.0.1")
COMMIT  ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
DATE    ?= $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
BINARY  := bb

LDFLAGS := -s -w \
  -X $(PKG).Version=$(VERSION) \
  -X $(PKG).Commit=$(COMMIT) \
  -X $(PKG).BuildDate=$(DATE)

.PHONY: build install test cover lint fmt clean help

build: ## Build binary
	@go build -trimpath -ldflags '$(LDFLAGS)' -o $(BINARY) ./cmd/bb

install: build ## Install to ~/.local/bin
	@cp $(BINARY) $(HOME)/.local/bin/$(BINARY)

test: ## Run all tests
	@go test ./...

cover: ## Show test coverage
	@go test -coverprofile=coverage.out ./internal/...
	@go tool cover -func=coverage.out
	@rm -f coverage.out

lint: ## Run go vet
	@go vet ./...

fmt: ## Format source files
	@gofmt -w ./cmd ./internal

clean: ## Remove built binary
	@rm -f $(BINARY)

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":[^:]*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
