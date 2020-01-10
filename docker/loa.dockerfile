FROM loalang/loa-base AS base

FROM debian

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

RUN mkdir -p /usr/local/var/log
RUN touch /usr/local/var/log/loa.log
RUN mkdir -p /usr/local/lib/loa

COPY std /usr/local/lib/loa/std

COPY --from=base /loalang/target/release/loa /usr/local/bin/loa

RUN mkdir /Project
WORKDIR /Project

ENTRYPOINT ["/usr/local/bin/loa"]
