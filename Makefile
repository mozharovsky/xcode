.PHONY: test test-rust test-js test-wasm build build-debug build-release build-wasm clean fmt lint check bench bench-rust bench-js

# Run all tests (Rust + JS + WASM)
test: test-rust test-js test-wasm

# Rust tests (no napi linking needed)
test-rust:
	cargo test --no-default-features

# JS integration tests (requires native binary)
test-js: build-debug
	npx ava __test__/index.spec.mjs

# WASM integration tests (requires: make build-wasm)
test-wasm:
	npx ava __test__/wasm.spec.mjs

# Build native binary (debug, fast)
build-debug:
	npx napi build --platform

# Build native binary (release, optimized)
build-release:
	npx napi build --platform --release

# Alias
build: build-release

# Build WASM package (single web target, WASM inlined as base64)
build-wasm:
	wasm-pack build --target web --out-dir pkg/web -- --no-default-features --features wasm
	./scripts/build-wasm-pkg.sh

# Check Rust compiles (fast, no codegen)
check:
	cargo check --no-default-features
	cargo check --features napi
	cargo check --no-default-features --features wasm --target wasm32-unknown-unknown

# Format
fmt:
	cargo fmt
	npx prettier --write "**/*.{js,mjs,ts,json}" --ignore-path .prettierignore

# Lint
lint:
	cargo clippy --no-default-features -- -D warnings

# Run all benchmarks
bench: bench-rust bench-js bench-wasm

# Pure Rust benchmark (no napi overhead)
bench-rust:
	cargo bench --no-default-features --bench parse_build

# JS benchmark: Rust vs TypeScript (requires: npm install @bacons/xcode)
bench-js: build-release
	node benches/benchmark.mjs

# WASM benchmark
bench-wasm:
	node benches/benchmark-wasm.mjs

# Clean all build artifacts
clean:
	cargo clean
	rm -rf *.node index.js index.d.ts pkg/
