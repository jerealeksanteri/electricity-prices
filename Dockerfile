# ---- Build stage ----
FROM rust:latest AS builder
WORKDIR /app

# dioxus-cli for `dx build`
RUN cargo install dioxus-cli@^0.6 --locked

COPY . .
# Builds web (wasm) assets + the server binary with the `server` feature.
RUN dx build --release --features server

# dx places build output under target/dx/<app>/release/web/. Normalize the
# server binary + public web assets into /app/out for the runtime stage.
#
# NOTE: The exact output layout (target/dx/voltti/release/web/)
# can vary by dioxus-cli patch version. The cp/find lines below are
# intentionally defensive: `|| true` prevents a layout mismatch from failing
# the build, and `find` locates the server binary by name + executable bit
# rather than assuming a hard path.
#
# After the first real `docker build`, inspect the build log for the actual
# paths and simplify these COPY/cp lines to remove the defensive fallbacks.
RUN mkdir -p /app/out \
    && cp -r target/dx/voltti/release/web/* /app/out/ 2>/dev/null || true \
    && find target -maxdepth 6 -type f -name voltti -perm -u+x -exec cp {} /app/out/server \; 2>/dev/null || true

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/out/ /app/
COPY --from=builder /app/assets/ /app/assets/

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["/app/server"]
