FROM rust:1-alpine3.18 as builder
RUN apk add --no-cache musl-dev
COPY . /build
WORKDIR /build
RUN cargo build --release

FROM alpine:3.18
ENV RUST_LOG=info
COPY --from=builder /build/target/release/proxy /app/proxy
CMD [ "/app/proxy", "-c", "/app/config.toml" ]
