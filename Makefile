all: clean build

install_wasm_opt:
	cargo install wasm-opt

build: install_worker_build
	cd soroflare-wrangler; worker-build --release

install_worker_build:
	cargo install worker-build

local: build
	cd soroflare-wrangler; wrangler dev --local

dev: build
	cd soroflare-wrangler; wrangler dev

clean:
	cargo clean

fmt:
	cargo fmt --all