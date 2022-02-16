FROM rust:buster as builder

RUN apt-get install libpq-dev
RUN mkdir -p /build
COPY ebina-bot /build/ebina-bot
COPY ebina-anilist /build/ebina-anilist
WORKDIR /build/ebina-bot
RUN cargo build --release


FROM rust:slim-buster
RUN apt-get update
RUN apt-get install -y libpq-dev
COPY --from=builder /build/ebina-bot/target/release/ebina-bot /usr/local/bin/ebina-bot
CMD ebina-bot