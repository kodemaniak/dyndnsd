FROM ekidd/rust-musl-builder:latest as builder

ADD --chown=rust:rust . ./

RUN cargo build --release

FROM alpine:3.16.0

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/dyndnsd /

WORKDIR /

USER 1000

ENTRYPOINT [ "./dyndnsd" ]