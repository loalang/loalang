.SILENT:

.PHONY: build install test debug docker/base docker/loa docker/vm docker/all docker/push

build:
	cargo build --release

debug:
	cargo build

test:
	cargo test --lib

install:
	cp target/release/loa /usr/local/bin/loa
	cp target/release/loavm /usr/local/bin/loavm

clean:
	rm /usr/local/bin/loa
	rm /usr/local/bin/loavm

docker/base:
	docker build -t loalang/base:latest -f docker/base.dockerfile .

docker/loa: docker/base
	docker build -t loalang/loa:latest -f docker/loa.dockerfile .

docker/vm: docker/base
	docker build -t loalang/vm:latest -f docker/vm.dockerfile .

docker/all: docker/loa docker/vm

docker/push: docker/all
	docker push loalang/loa
	docker push loalang/vm
