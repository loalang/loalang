FROM loalang/base AS base

RUN rustup target add x86_64-unknown-linux-musl

RUN cargo build --bin=loavm --features=build-bin-vm --release --target=x86_64-unknown-linux-musl

FROM scratch

COPY --from=base /loalang/target/x86_64-unknown-linux-musl/release/loavm /loavm

ENTRYPOINT ["/loavm"]
