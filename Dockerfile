FROM rust:1.79 as builder

WORKDIR /usr/src/traidano
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm

RUN apt update && apt upgrade && apt install -y openssl ca-certificates

COPY --from=builder /usr/src/traidano/target/release/traidano /usr/local/bin/traidano

RUN update-ca-certificates

CMD ["traidano"]