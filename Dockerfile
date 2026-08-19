FROM rust:1.97-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM scratch
COPY --from=builder /build/target/release/immich-tools /immich-tools
ENTRYPOINT ["/immich-tools"]
