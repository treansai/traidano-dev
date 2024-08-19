FROM rust:1.79 as builder

WORKDIR /usr/src/traidano
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm

RUN apt update && apt upgrade
RUN apt install -y openssl

COPY --from=builder /usr/src/traidano/target/release/traidano /usr/local/bin/traidano

CMD ["traidano"]