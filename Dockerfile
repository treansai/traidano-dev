FROM rust:1.69.0 as builder

WORKDIR /usr/src/traidano
COPY . .
RUN cargo build --release

FROM debian:buster-slim

COPY --from /usr/src/traidano/target/release/traidano /usr/local/bin/traidano
CMD ["traidano"]