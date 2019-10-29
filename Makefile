.SILENT:

.PHONY: build install test debug

build:
	cargo build --release

debug:
	cargo build

test:
	cargo test --lib

install:
	cp target/release/loa /usr/local/bin/loa

clean:
	rm /usr/local/bin/loa
