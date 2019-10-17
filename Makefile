.SILENT:

.PHONY: build install test

build:
	cargo build --release

test:
	cargo test --lib

install:
	cp target/release/loa /usr/local/bin/loa

clean:
	rm /usr/local/bin/loa
