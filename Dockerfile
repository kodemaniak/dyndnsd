FROM rust:1.71.1 as builder

ADD . .

WORKDIR /

RUN cargo build --release

FROM alpine:3.18.2

COPY --from=builder /target/release/dyndnsd /

WORKDIR /

USER 1000

ENTRYPOINT [ "./dyndnsd" ]
