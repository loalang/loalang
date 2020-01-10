FROM loalang/vm-base AS base

FROM scratch

COPY --from=base /loalang/target/x86_64-unknown-linux-musl/release/loavm /loavm

ENTRYPOINT ["/loavm"]
