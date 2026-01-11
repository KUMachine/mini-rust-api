# ===========================
# 🚧 Build Stage
# ===========================
FROM rust:1.85-slim AS builder

# Install build dependencies including OpenSSL
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      gcc libssl-dev pkg-config make ca-certificates openssl \
      libssl3 libcrypto3 \
 && rm -rf /var/lib/apt/lists/*

# Set pkg-config to find OpenSSL
ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
ENV OPENSSL_STATIC=0

# Optimize for faster builds
ENV CARGO_INCREMENTAL=1

# Create a new empty shell project
WORKDIR /app

# Copy dependency files and migration folder first (for better caching)
COPY Cargo.toml ./
COPY migration/ ./migration/

# Create minimal source structure for dependency resolution
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/lib.rs

# Pre-download and cache dependencies (this layer will be cached if Cargo.toml/lock haven't changed)
RUN cargo fetch

# Remove dummy files and copy actual source code
RUN rm -rf src
COPY src/ ./src/

# Build the actual application (this layer will only rebuild if source code changes)
RUN cargo build --release


# ===========================
# 🏁 Final Stage (Minimal Runtime)
# ===========================
FROM scratch AS runtime

# Copy the binary from the builder
COPY --from=builder /app/target/release/rust-mini-api /usr/src/app

# Set mimalloc optimization environment variables
ENV MIMALLOC_SHOW_STATS=1
ENV MIMALLOC_PAGE_RESET=1
ENV MIMALLOC_SECURE=1
ENV MIMALLOC_LARGE_OS_PAGES=1
ENV MIMALLOC_RESERVE_HUGE_OS_PAGES=1
ENV MIMALLOC_EAGER_COMMIT=1
ENV MIMALLOC_EAGER_REGION_COMMIT=1

# Copy necessary shared libraries for glibc
COPY --from=builder /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libdl.so.2 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/librt.so.1 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libpthread.so.0 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libssl.so.3 /lib/x86_64-linux-gnu/
COPY --from=builder /lib64/ld-linux-x86-64.so.2 /lib64/
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Set binary as entrypoint
ENTRYPOINT ["/usr/src/app"]