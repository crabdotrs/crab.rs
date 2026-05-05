# AI WRITTEN (chat.qwen.ai) not verified

FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

COPY crab.rs/Cargo.toml crab.rs/Cargo.lock ./
COPY crab.rs/crab-core ./crab-core
COPY crab.rs/crab-lexer ./crab-lexer
COPY crab.rs/crab-parser ./crab-parser
COPY crab.rs/crab-codegen ./crab-codegen
COPY crab.rs/crab-ffi ./crab-ffi
COPY crab.rs/crab-cli ./crab-cli

RUN cargo build --release -p crab-cli

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    gcc \
    libc6-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/crab /usr/local/bin/crab

WORKDIR /workspace

ENTRYPOINT ["crab"]
CMD ["--help"]
