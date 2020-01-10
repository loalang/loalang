FROM loalang/base

COPY Cargo.toml Cargo.lock ./

# This is an unfortunate step, but needed for us
# to be able to install dependencies before
# copying files into the container.
# {{{
RUN mkdir -p src/lib
RUN touch src/lib/mod.rs
RUN mkdir -p src/bin
RUN echo "fn main() {}" > src/bin/loa.rs
# }}}

RUN cargo build --bin=loa --release --features build-bin-loa

RUN rm -rf src \
  target/release/deps/loa-* \
  target/release/deps/libloa-*

COPY . .

RUN cargo build --bin=loa --release --features build-bin-loa
