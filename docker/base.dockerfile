FROM rust

RUN rustup default nightly

RUN mkdir /loalang
WORKDIR /loalang
