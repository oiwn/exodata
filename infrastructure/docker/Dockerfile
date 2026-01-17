FROM rust:latest as builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install --locked cargo-leptos
RUN rustup target add wasm32-unknown-unknown

RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 \
    && chmod +x tailwindcss-linux-x64 \
    && mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
COPY style ./style
COPY public ./public
COPY tailwind.config.js ./

RUN cargo leptos build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/server/release/exoplanets-catalog /app/
COPY --from=builder /app/target/site /app/site

EXPOSE 3000
CMD ["/app/exoplanets-catalog"]
