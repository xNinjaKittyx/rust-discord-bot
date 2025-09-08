FROM rust:1.88 AS base
RUN apt update && apt upgrade -y && apt install -y cmake && apt clean && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --version ^0.1
RUN cargo install sccache --version ^0.7
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache


FROM base AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release

FROM ubuntu:24.04
WORKDIR /app

RUN apt update && apt upgrade -y && apt install -y curl && apt clean && rm -rf /var/lib/apt/lists/*
RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/bin/yt-dlp && chmod a+rx /usr/bin/yt-dlp

COPY --from=builder /app/target/release/rust-discord-bot rust-discord-bot

CMD ["/app/rust-discord-bot"]
