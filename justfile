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
    cd examples/c_enum && cargo run --package azadi -- status.md --gen .

# ── Packaging ─────────────────────────────────────────────────────────────────

# Build container stage: glibc | musl | windows | fedora
build-container TARGET:
    podman build --target {{TARGET}} -t azadi-{{TARGET}} .

# Build container and export artifacts into dist/TARGET/
export TARGET: (build-container TARGET)
    mkdir -p dist/{{TARGET}}
    podman create --name azadi-export-{{TARGET}} azadi-{{TARGET}}
    podman cp azadi-export-{{TARGET}}:/out/. dist/{{TARGET}}/
    podman rm azadi-export-{{TARGET}}

# Build and export all targets
export-all: (export "glibc") (export "musl") (export "windows") (export "fedora")

# Build .deb locally (requires cargo-deb)
deb:
    cargo build --release --workspace
    cargo deb -p azadi --no-build

# Build .rpm locally (requires cargo-generate-rpm)
rpm:
    cargo build --release --workspace
    cargo generate-rpm -p crates/azadi

# Tag, wait for CI, then publish PKGBUILD + flake.nix + AUR (usage: just tag v0.2.0)
tag VERSION:
    python packaging/update_release.py --tag {{VERSION}}

# Re-tag HEAD, wait for CI, then publish (usage: just re-tag v0.2.0)
re-tag VERSION:
    -git push --delete origin {{VERSION}}
    -git tag -d {{VERSION}}
    python packaging/update_release.py --tag {{VERSION}}

# Re-run publish only — tag already pushed and CI already done (usage: just update-release 0.2.0)
update-release VERSION:
    python packaging/update_release.py {{VERSION}}

# ── Clean ─────────────────────────────────────────────────────────────────────

# cargo clean + dist/
clean:
    cargo clean
    rm -rf dist/
