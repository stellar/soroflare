all: clean build

install_wasm_opt:
	cargo install wasm-opt

build: install_worker_build
	cd wrangler; worker-build --release

install_worker_build:
# this version has a bug fix we require https://github.com/cloudflare/workers-rs/issues/255
	cargo install --git https://github.com/Smephite/workers-rs.git 

local: build
	cd wrangler; wrangler dev --local

dev: build
	cd wrangler; wrangler dev

clean:
	cargo clean

fmt:
	cargo fmt --all