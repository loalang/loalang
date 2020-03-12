FROM loalang/loa-base AS base

FROM debian

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

RUN mkdir /sdk
ENV LOA_SDK /sdk
WORKDIR /sdk

RUN mkdir docs
COPY src/bin/docs/public docs/html

COPY std std
RUN rm -rf std/.git

RUN mkdir bin
COPY --from=base /loalang/target/release/loa bin/loa

RUN mkdir log
RUN touch log/loa.log

RUN mkdir /Project
WORKDIR /Project

ENTRYPOINT ["/usr/local/bin/loa"]
