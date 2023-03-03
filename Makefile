all: clean build

install_wasm_opt:
	cargo install wasm-opt

build: install_worker_build
	worker-build --release

install_worker_build:
	cargo install worker-build

local: build
	wrangler dev --local

dev: build
	wrangler dev

clean:
	cargo clean

fmt:
	cargo fmt --all