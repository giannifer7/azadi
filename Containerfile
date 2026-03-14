# Containerfile — multi-stage packaging builds for azadi
#
# Stages:
#   glibc   — Debian binary + .deb  (cargo-deb)  — includes python feature
#   musl    — Alpine static binary               — no python (PyO3 + musl unsupported)
#   windows — MinGW cross-compiled .exe          — no python (cross-compiling PyO3 unsupported)
#   fedora  — Fedora binary + .rpm               — includes python feature
#
# Usage:
#   podman build --target glibc  -t azadi-glibc  .
#   podman build --target fedora -t azadi-fedora .

# ── Rust base (Debian bookworm) ───────────────────────────────────────────────
FROM debian:bookworm-slim AS rust-base

RUN apt-get update && apt-get install -y --no-install-recommends \
        curl ca-certificates build-essential pkg-config git \
        python3 python3-dev \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf \
    | sh -s -- -y --default-toolchain stable --no-modify-path

# ── cargo-chef planner ────────────────────────────────────────────────────────
FROM rust-base AS planner
WORKDIR /src
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── dependency cacher (glibc / deb) ──────────────────────────────────────────
FROM rust-base AS cacher
WORKDIR /src
RUN cargo install cargo-chef cargo-deb cargo-generate-rpm
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ── glibc: Debian binary + .deb ──────────────────────────────────────────────
FROM cacher AS glibc
COPY . .
RUN cargo build --release --workspace
RUN cargo deb -p azadi --no-build
RUN mkdir -p /out \
    && cp target/release/azadi        /out/ \
    && cp target/release/azadi-macros /out/ \
    && cp target/release/azadi-noweb  /out/ \
    && cp target/debian/*.deb         /out/

# ── musl: static binary ───────────────────────────────────────────────────────
# PyO3 requires libpython which is not available for musl targets.
FROM rust-base AS musl
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --no-default-features --target x86_64-unknown-linux-musl --workspace
RUN mkdir -p /out \
    && cp target/x86_64-unknown-linux-musl/release/azadi        /out/ \
    && cp target/x86_64-unknown-linux-musl/release/azadi-macros /out/ \
    && cp target/x86_64-unknown-linux-musl/release/azadi-noweb  /out/

# ── windows: MinGW cross-compilation ─────────────────────────────────────────
# PyO3 cross-compilation to Windows requires a Windows Python SDK; not supported here.
FROM rust-base AS windows
RUN apt-get update && apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
WORKDIR /src
COPY . .
RUN rustup target add x86_64-pc-windows-gnu \
    && cargo build --release --no-default-features --target x86_64-pc-windows-gnu --workspace
RUN mkdir -p /out \
    && cp target/x86_64-pc-windows-gnu/release/azadi.exe        /out/ \
    && cp target/x86_64-pc-windows-gnu/release/azadi-macros.exe /out/ \
    && cp target/x86_64-pc-windows-gnu/release/azadi-noweb.exe  /out/

# ── fedora: RPM ───────────────────────────────────────────────────────────────
FROM fedora:latest AS fedora
RUN dnf install -y curl gcc pkg-config git python3 python3-devel && dnf clean all
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl https://sh.rustup.rs -sSf \
    | sh -s -- -y --default-toolchain stable --no-modify-path
RUN cargo install cargo-generate-rpm
WORKDIR /src
COPY . .
RUN cargo build --release --workspace
RUN cargo generate-rpm -p azadi
RUN mkdir -p /out \
    && cp target/release/azadi        /out/ \
    && cp target/release/azadi-macros /out/ \
    && cp target/release/azadi-noweb  /out/ \
    && cp target/generate-rpm/*.rpm   /out/
