run: build
	RUST_LOG=info ./target/release/query_cache

build:
	cargo build --release

format:
	rustfmt src/**/*.rs --edition 2021

