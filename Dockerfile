# syntax=docker/dockerfile:1

# ===========================
# Build Stage
# ===========================
FROM rust:1.99-slim-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migration/ ./migration/
COPY src/ ./src/

# Cache the cargo registry and target dir between builds; copy the binaries
# out because cache mounts are not part of the image
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --bin mini-rust-api --bin migration \
 && mkdir -p /out \
 && cp target/release/mini-rust-api target/release/migration /out/


# ===========================
# Runtime Stage
# ===========================
# distroless/cc ships glibc and libgcc only; TLS uses rustls with bundled roots
FROM gcr.io/distroless/cc-debian13:nonroot AS runtime

COPY --from=builder /out/mini-rust-api /usr/local/bin/mini-rust-api
COPY --from=builder /out/migration /usr/local/bin/migration

ENV SERVER__HOST=0.0.0.0 \
    SERVER__PORT=3000

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/mini-rust-api"]
