FROM rust:1.59.0-buster as builder

RUN apt-get install libpq-dev
RUN mkdir -p /build
COPY ./ /build
WORKDIR /build/ebina-bot
RUN cargo build --release


FROM rust:slim-buster
RUN apt-get update
RUN apt-get install -y libpq-dev libcurl4-openssl-dev
COPY --from=builder /build/target/release/ebina-bot /usr/local/bin/ebina-bot
CMD ebina-bot