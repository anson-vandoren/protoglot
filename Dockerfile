FROM rust:latest as builder

RUN USER=root cargo new --bin protoglot
WORKDIR /protoglot

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./config ./config

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
COPY --from=builder /protoglot/target/x86_64-unknown-linux-musl/release/protoglot /usr/local/bin/protoglot
COPY ./entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]