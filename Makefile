.SILENT:

.PHONY: build install test debug docker/base docker/loa docker/vm docker/all docker/push dist dist/macos dist/linux _dist _dist/build version

VERSION=$(shell toml get Cargo.toml 'package.version' | jq -r)

build:
	cargo build --release --features build-binary

debug:
	cargo build --features build-binary

version:
	echo $(VERSION)

test:
	RUST_BACKTRACE=1 cargo test --lib
	RUST_BACKTRACE=1 cargo test --features=build-binary --bin loa

install:
	git submodule init
	git submodule update
	cp target/release/loa /usr/local/bin/loa
	cp target/release/loavm /usr/local/bin/loavm
	mkdir -p /usr/local/lib/loa/std
	rm -rf /usr/local/lib/loa/std
	cp -r std /usr/local/lib/loa/std
	rm -rf /usr/local/lib/loa/docs-html
	cp -r src/bin/docs/public /usr/local/lib/loa/docs-html
	mkdir -p /usr/local/var/log
	touch /usr/local/var/log/loa.log
	chmod 777 /usr/local/var/log/loa.log

clean:
	rm /usr/local/bin/loa
	rm /usr/local/bin/loavm
	rm -rf /usr/local/lib/loa

docker/base:
	docker build -t loalang/base:latest -f docker/base.dockerfile .

docker/loa: docker/base
	docker build -t loalang/loa:$(VERSION) -t loalang/loa:latest -f docker/loa.dockerfile .

docker/vm: docker/base
	docker build -t loalang/vm:$(VERSION) -t loalang/vm:latest -f docker/vm.dockerfile .

docker/all: docker/loa docker/vm

docker/push: docker/all
	docker push loalang/loa:latest
	docker push loalang/loa:$(VERSION)
	docker push loalang/vm:latest
	docker push loalang/vm:$(VERSION)

dist: dist/macos dist/linux
	echo "# Published loa v$(VERSION)"
	echo "# MacOS"
	echo "sha256: $(shell shasum -a 256 target/dist/$(VERSION)_x86_64-macos.tar.gz | awk '{ print $$1 }')"
	echo "archive: https://storage.googleapis.com/loalang-releases/$(VERSION)_x86_64-macos.tar.gz"
	echo "# Linux"
	echo "sha256: $(shell shasum -a 256 target/dist/$(VERSION)_x86_64-linux.tar.gz | awk '{ print $$1 }')"
	echo "archive: https://storage.googleapis.com/loalang-releases/$(VERSION)_x86_64-linux.tar.gz"

dist/macos:
	DIST_NAME=x86_64-macos TARGET_TRIPLE=x86_64-apple-darwin make _dist
	gsutil cp target/dist/$(VERSION)_x86_64-macos.tar.gz gs://loalang-releases/

dist/linux: docker/base
	docker run --rm -v $(PWD)/target:/loalang/target -w /loalang -e DIST_NAME=x86_64-linux -e TARGET_TRIPLE=x86_64-unknown-linux-gnu loalang/base make _dist
	gsutil cp target/dist/$(VERSION)_x86_64-linux.tar.gz gs://loalang-releases/

_dist:
	rm -rf target/dist/$(VERSION)/$(DIST_NAME)
	mkdir -p target/dist/$(VERSION)/$(DIST_NAME)/bin
	mkdir -p target/dist/$(VERSION)/$(DIST_NAME)/lib/loa
	mkdir -p target/dist/$(VERSION)/$(DIST_NAME)/var/log
	touch target/dist/$(VERSION)/$(DIST_NAME)/var/log/loa.log
	cp -r src/bin/docs/public target/dist/$(VERSION)/$(DIST_NAME)/lib/loa/docs-html
	cargo build --release --target $(TARGET_TRIPLE) --features build-binary
	cp target/$(TARGET_TRIPLE)/release/loa target/dist/$(VERSION)/$(DIST_NAME)/bin/loa
	cp target/$(TARGET_TRIPLE)/release/loavm target/dist/$(VERSION)/$(DIST_NAME)/bin/loavm
	cp -r std target/dist/$(VERSION)/$(DIST_NAME)/lib/loa/std
	rm -rf target/dist/$(VERSION)/$(DIST_NAME)/lib/loa/std/.git
	mv target/dist/$(VERSION)/$(DIST_NAME) loa
	tar -czf target/dist/$(VERSION)_$(DIST_NAME).tar.gz loa
	mv loa target/dist/$(VERSION)/$(DIST_NAME)
