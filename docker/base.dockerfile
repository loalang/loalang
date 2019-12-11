FROM rust

RUN apt-get update && apt-get install -y libssl-dev

RUN rustup default nightly
# RUN rustup target add x86_64-unknown-linux-musl

RUN mkdir /loalang
WORKDIR /loalang

COPY Cargo.toml Cargo.lock ./

# This is an unfortunate step, but needed for us
# to be able to install dependencies before
# copying files into the container.
# {{{
RUN mkdir -p src/lib
RUN touch src/lib/mod.rs
RUN mkdir -p src/bin
RUN echo "fn main() {}" > src/bin/loa.rs
RUN echo "fn main() {}" > src/bin/loavm.rs
# }}}

# RUN cargo build --release --target=x86_64-unknown-linux-musl --features build-binary
RUN cargo build --release --target=x86_64-unknown-linux-gnu --features build-binary

# RUN rm -rf src \
#   target/x86_64-unknown-linux-musl/release/deps/loa-* \
#   target/x86_64-unknown-linux-musl/release/deps/libloa-*
RUN rm -rf src \
  target/x86_64-unknown-linux-gnu/release/deps/loa-* \
  target/x86_64-unknown-linux-gnu/release/deps/libloa-*

COPY . .
