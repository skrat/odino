all: web

build:
	@cargo build \
	--profile web \
	--target wasm32-unknown-unknown \
	--no-default-features \
	--features web \

	@~/.cargo/bin/wasm-bindgen \
	--no-typescript \
	--target web \
	--out-dir ./out/ \
	--out-name odino \
	target/wasm32-unknown-unknown/web/odino.wasm

web: build
	@wasm-opt -Oz \
	--enable-reference-types \
	--enable-simd \
	--enable-bulk-memory \
	-o odino_bg.wasm_opt \
	out/odino_bg.wasm

	@mv odino_bg.wasm_opt out/odino_bg.wasm
	@cp index.html out/
