FROM loalang/base AS base

RUN cargo build --bin=loa --release --target=x86_64-unknown-linux-musl
RUN mkdir /Project

FROM alpine

COPY --from=base /loalang/target/x86_64-unknown-linux-musl/release/loa /usr/local/bin/loa
COPY --from=base /Project /Project

WORKDIR /Project

ENTRYPOINT ["/usr/local/bin/loa"]
