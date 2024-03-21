FROM rust:1.77.0 as builder

ADD . .

WORKDIR /

RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /target/release/dyndnsd /

WORKDIR /

USER 1000

CMD [ "/dyndnsd" ]
