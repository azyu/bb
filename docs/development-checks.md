# Development Checks Pattern

This repository uses a small, reusable pattern for deterministic local Rust checks.

## Goals
- Pin the Rust toolchain so local and CI behavior stay aligned.
- Run fast, deterministic checks before commit.
- Reuse the same commands in local hooks, `Makefile`, and CI.
- Avoid extra runtime or package-manager dependencies for hook execution.

## Pieces

### 1. Pin the toolchain
Use `rust-toolchain.toml` at the repository root:

```toml
[toolchain]
channel = "1.93.0"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

### 2. Standardize local commands
Expose one command per concern in `Makefile`:

```make
fmt:        cargo fmt --manifest-path rust/Cargo.toml --all
fmt-check:  cargo fmt --manifest-path rust/Cargo.toml --all --check
lint:       cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
test:       cargo test --manifest-path rust/Cargo.toml
hooks-install:
	git config core.hooksPath .githooks
```

### 3. Commit repo-managed hooks
Store hook scripts in `.githooks/` and point Git at that directory.

`pre-commit`:

```sh
#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

make fmt-check
make lint
```

`pre-push`:

```sh
#!/bin/sh
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

make test
```

Install once per clone:

```bash
make hooks-install
```

### 4. Mirror the same commands in CI
Keep CI as the enforcement backstop and run the same commands:

```yaml
- run: make fmt-check
- run: make lint
- run: make test
```

## Why this pattern
- Smaller than adding a separate hook manager.
- Easy to copy to another Rust repository.
- Deterministic because `rust-toolchain.toml` pins the compiler and components.
- Reviewable because hook behavior is tracked in the repository.

## Copy Checklist
1. Add `rust-toolchain.toml`.
2. Add `fmt`, `fmt-check`, `lint`, `test`, and `hooks-install`.
3. Add `.githooks/pre-commit` and `.githooks/pre-push`.
4. Run `chmod +x .githooks/pre-commit .githooks/pre-push`.
5. Run `make hooks-install`.
6. Update CI to run the same commands.
