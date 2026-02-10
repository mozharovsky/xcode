.PHONY: test test-rust test-js build build-debug build-release clean fmt lint check bench bench-rust bench-js

# Run all tests (Rust + JS)
test: test-rust test-js

# Rust tests (no napi linking needed)
test-rust:
	cargo test --no-default-features

# JS integration tests (requires native binary)
test-js: build-debug
	npx ava __test__/index.spec.mjs

# Build native binary (debug, fast)
build-debug:
	npx napi build --platform

# Build native binary (release, optimized)
build-release:
	npx napi build --platform --release

# Alias
build: build-release

# Check Rust compiles (fast, no codegen)
check:
	cargo check --no-default-features
	cargo check --features napi

# Format
fmt:
	cargo fmt

# Lint
lint:
	cargo clippy --no-default-features -- -D warnings

# Run all benchmarks
bench: bench-rust bench-js

# Pure Rust benchmark (no napi overhead)
bench-rust:
	cargo bench --no-default-features --bench parse_build

# JS benchmark: Rust vs TypeScript (requires: npm install @bacons/xcode)
bench-js: build-release
	node benches/benchmark.mjs

# Clean all build artifacts
clean:
	cargo clean
	rm -f *.node index.js index.d.ts
