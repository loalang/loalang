.SILENT:

.PHONY: build install

build:
	cargo build --release

install:
	cp target/release/loa /usr/local/bin/loa
