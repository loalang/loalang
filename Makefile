.SILENT:

.PHONY: build install test debug docker/base docker/loa docker/vm docker/all docker/push

build:
	cargo build --release --features build-binary

debug:
	cargo build --features build-binary

test:
	cargo test --lib

install:
	cp target/release/loa /usr/local/bin/loa
	cp target/release/loavm /usr/local/bin/loavm
	mkdir -p /usr/local/lib/loa/std
	rm -rf /usr/local/lib/loa/std
	cp -r std /usr/local/lib/loa/std
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
	docker build -t loalang/loa:latest -f docker/loa.dockerfile .

docker/vm: docker/base
	docker build -t loalang/vm:latest -f docker/vm.dockerfile .

docker/all: docker/loa docker/vm

docker/push: docker/all
	docker push loalang/loa
	docker push loalang/vm
