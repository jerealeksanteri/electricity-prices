# ---- Build stage ----
FROM rust:latest AS builder
WORKDIR /app

# dioxus-cli for `dx build`, plus the wasm target for the client build.
RUN cargo install dioxus-cli@^0.6 --locked \
    && rustup target add wasm32-unknown-unknown

COPY . .
# Fullstack build: dx builds the wasm client (with the `web` feature) and the
# Axum server (with the `server` feature) as SEPARATE targets and picks the
# right feature for each automatically. Do NOT pass `--features server` here —
# it would force `server` (tokio/mio) onto the wasm client, which can't compile
# for wasm32.
RUN dx build --release --fullstack

# A dx fullstack build emits the server binary (named `server`) and the client
# `public/` assets together under target/dx/<app>/release/web/. The server
# serves `./public` relative to its own working directory.

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# ca-certificates for TLS trust roots; libssl3 because reqwest (via the entsoe
# crate) links OpenSSL by default.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy `server` + `public/` (which already includes tailwind.css + echarts.min.js).
COPY --from=builder /app/target/dx/voltti/release/web/ /app/

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["/app/server"]
