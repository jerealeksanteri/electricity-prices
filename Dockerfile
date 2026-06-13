# syntax=docker/dockerfile:1
# ---- Build stage ----
FROM rust:bookworm AS builder
WORKDIR /app

# Install dx as a PREBUILT binary via cargo-binstall (seconds) instead of
# compiling dioxus-cli from source (minutes); fall back to a source install if
# no prebuilt is available. Add the wasm target for the client build.
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && ( cargo binstall -y dioxus-cli@^0.6 || cargo install dioxus-cli@^0.6 --locked ) \
    && rustup target add wasm32-unknown-unknown

COPY . .
# Fullstack build: dx builds the wasm client (with the `web` feature) and the
# Axum server (with the `server` feature) as SEPARATE targets and picks the
# right feature for each automatically. Do NOT pass `--features server` here —
# it would force `server` (tokio/mio) onto the wasm client, which can't compile
# for wasm32.
#
# BuildKit cache mounts persist the cargo registry + target dir across builds
# (big speedup on repeat builds). The target dir is a cache mount and is NOT
# committed to the image, so copy the output into a real dir (/app/dist) after.
# A dx fullstack build emits the server binary (named `server`) and the client
# `public/` assets together under target/dx/<app>/release/web/.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    dx build --release --fullstack \
    && mkdir -p /app/dist \
    && cp -r target/dx/voltti/release/web /app/dist/web

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# ca-certificates for TLS trust roots; libssl3 because reqwest (via the entsoe
# crate) links OpenSSL by default.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy `server` + `public/` (which already includes tailwind.css + echarts.min.js).
COPY --from=builder /app/dist/web/ /app/

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["/app/server"]
