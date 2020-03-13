.SILENT:

VERSION ?= $(shell cargo pkgid | awk -F\# '{print $$2}')
LOA_SDK ?= sdk

.PHONY: build
build:
	cargo build --release --features build-bin-loa,build-bin-vm

.PHONY: debug
debug:
	cargo build --features build-bin-loa,build-bin-vm

.PHONY: version
version:
	echo $(VERSION)

.PHONY: test
test:
	cargo test --features=test-library --lib -- --nocapture
	cargo test --features=test-library,build-bin-loa --bin loa -- --nocapture

.PHONY: install
install: clean
	mkdir $(LOA_SDK)
	mkdir $(LOA_SDK)/docs
	cp -r src/bin/docs/public $(LOA_SDK)/docs/html
	cp -r std $(LOA_SDK)/std
	rm -rf $(LOA_SDK)/std/.git
	mkdir $(LOA_SDK)/bin
	cp target/release/loa target/release/loavm $(LOA_SDK)/bin/
	mkdir $(LOA_SDK)/log
	touch $(LOA_SDK)/log/loa.log
	chmod 777 $(LOA_SDK)/log/loa.log

.PHONY: clean
clean:
	rm -rf $(LOA_SDK)
