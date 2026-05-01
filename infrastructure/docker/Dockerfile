FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-binstall for faster installation
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install cargo-leptos using binstall (much faster than cargo install)
RUN cargo binstall --no-confirm cargo-leptos
RUN rustup target add wasm32-unknown-unknown

# Install Tailwind CSS (detect architecture)
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ]; then \
        curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-arm64 && \
        chmod +x tailwindcss-linux-arm64 && \
        mv tailwindcss-linux-arm64 /usr/local/bin/tailwindcss; \
    else \
        curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 && \
        chmod +x tailwindcss-linux-x64 && \
        mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss; \
    fi

WORKDIR /app
ARG BUILD_TIMESTAMP=unknown
ENV BUILD_TIMESTAMP=${BUILD_TIMESTAMP}
ARG BUILD_DATE=
ENV BUILD_DATE=${BUILD_DATE}
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY docs ./docs
COPY src ./src
COPY style ./style
COPY public ./public

RUN cargo leptos build --release --split

# Normalize the server binary name for the runtime image. cargo-leptos has used
# different target names as the package and public command names changed.
RUN mkdir -p /app/server-bin && \
    if [ -x /app/target/release/exodata ]; then \
        cp /app/target/release/exodata /app/server-bin/exodata; \
    elif [ -x /app/target/release/exoplanets-catalog ]; then \
        cp /app/target/release/exoplanets-catalog /app/server-bin/exodata; \
    else \
        echo "Server binary not found" >&2; \
        ls -lah /app/target/release/ >&2; \
        exit 1; \
    fi

# Debug: List where files actually are
RUN echo "=== Listing target/release ===" && \
    ls -lah /app/target/release/ | grep -E 'exodata|exoplanets' || true && \
    echo "=== Listing target/server/release ===" && \
    ls -lah /app/target/server/release/ 2>/dev/null || echo "Directory does not exist" && \
    echo "=== Listing target/site ===" && \
    ls -lah /app/target/site/

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/server-bin/exodata /app/exodata
COPY --from=builder /app/target/site /app/site

EXPOSE 3000
CMD ["/app/exodata"]
