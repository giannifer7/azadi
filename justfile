# justfile — azadi workspace

# Default: list available recipes
default:
    @just --list

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the whole workspace (debug)
build:
    cargo build

# Build the whole workspace (release)
release:
    cargo build --release

# ── Test ──────────────────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test

# Run tests for azadi-macros only
test-macros:
    cargo test --package azadi-macros

# Run tests for azadi-noweb only
test-noweb:
    cargo test --package azadi-noweb

# ── Lint ──────────────────────────────────────────────────────────────────────

# Clippy (warnings as errors)
lint:
    cargo clippy -- -D warnings

# Format check
fmt-check:
    cargo fmt --check

# Apply formatting
fmt:
    cargo fmt

# ── Run ───────────────────────────────────────────────────────────────────────

# Run the combined azadi tool (usage: just azadi src/foo.md)
azadi FILE:
    cargo run --package azadi -- "{{FILE}}"

# Run azadi-macros on a file (usage: just macros src/foo.md)
macros FILE:
    cargo run --package azadi-macros -- "{{FILE}}"

# Run azadi-noweb on a file (usage: just noweb src/foo.md)
noweb FILE:
    cargo run --package azadi-noweb -- "{{FILE}}"

# ── Examples ──────────────────────────────────────────────────────────────────

# Regenerate the c_enum example
example-c-enum:
    cd examples/c_enum && cargo run --package azadi -- status.md --gen src

# ── Clean ─────────────────────────────────────────────────────────────────────

# cargo clean
clean:
    cargo clean
