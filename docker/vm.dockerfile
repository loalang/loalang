FROM loalang/base AS base

RUN cargo build --bin=loavm --release --target=x86_64-unknown-linux-musl

FROM scratch

COPY --from=base /loalang/target/x86_64-unknown-linux-musl/release/loavm /loavm

ENTRYPOINT ["/loavm"]
