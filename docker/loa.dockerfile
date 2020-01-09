FROM loalang/base AS base

RUN cargo build --bin=loa --release --features build-bin-loa
RUN mkdir /Project

FROM alpine

RUN mkdir -p /usr/local/var/log
RUN touch /usr/local/var/log/loa.log
RUN mkdir -p /usr/local/lib/loa

COPY std /usr/local/lib/loa/std

COPY --from=base /loalang/target/release/loa /usr/local/bin/loa
COPY --from=base /Project /Project

WORKDIR /Project

ENTRYPOINT ["/usr/local/bin/loa"]
