FROM rust:1.71.1-slim-bookworm as builder
WORKDIR /opt/llm_router

RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install dependencies
COPY ./src /opt/llm_router/src
COPY ./Cargo.toml /opt/llm_router/Cargo.toml
COPY ./Cargo.lock /opt/llm_router/Cargo.lock

RUN chown -R 1000:1000 /opt/llm_router

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssl \
    libc6 \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /opt/llm_router/target/release/llm_router /usr/local/bin/llm_router

ENTRYPOINT [ "llm_router" ]