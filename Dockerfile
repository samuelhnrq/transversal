FROM rust:1.88-alpine AS builder

RUN addgroup -S rust -g 1000 && adduser -u 1000 rust -G rust -S -h /home/rust/ && \
    apk add --no-cache musl-dev && \
    rustup component add rustfmt clippy
USER rust
WORKDIR /home/rust/app

COPY --chown=rust:rust . .
RUN --mount=type=cache,target=/home/rust/.cargo/registry,uid=1000,gid=1000 \
    --mount=type=cache,target=./target,uid=1000,gid=1000 \
    cargo fmt --all --check && \
    cargo test --all && \
    cargo build --release && \
    cargo clippy --release -- -D warnings && \
    cp target/release/transversal /home/rust/

FROM alpine:latest

COPY --from=builder /home/rust/transversal /usr/local/bin/transversal

CMD [ "/usr/local/bin/transversal" ]
