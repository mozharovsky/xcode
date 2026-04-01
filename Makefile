.PHONY: test build clean fmt lint bench

test:
	cargo test

build:
	cargo build --release

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

bench:
	cargo bench --bench parse_build

clean:
	cargo clean
