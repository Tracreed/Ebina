FROM rust:1.59.0-buster as builder

RUN apt-get install libpq-dev
RUN mkdir -p /build
COPY ./ /build
WORKDIR /build/ebina-bot
RUN cargo build --release


FROM rust:slim-buster
RUN apt-get update
RUN apt-get install -y libpq-dev libcurl4-openssl-dev
RUN cargo install --locked trunk
RUN install wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown
COPY --from=builder /build/ebina-bot/target/release/ebina-bot /usr/local/bin/ebina-bot
CMD ebina-bot