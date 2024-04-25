FROM rust:1.77

RUN apt update && apt upgrade -y && apt install -y cmake && apt clean && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/bot
COPY . .

RUN cargo install --path .

CMD ["rust-discord-bot"]