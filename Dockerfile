FROM rust:1.71.1 as builder

ADD . .

WORKDIR /

RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /target/release/dyndnsd /

WORKDIR /

USER 1000

CMD [ "/dyndnsd" ]
