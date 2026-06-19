.PHONY: build run dev-front dev-back check clean

build:
	cd crates/frontend && trunk build --release
	cargo build --release -p backend

run: build
	cargo run --release -p backend

dev-back:
	cargo run -p backend

dev-front:
	cd crates/frontend && trunk serve

check:
	cargo check --workspace

clean:
	cargo clean && rm -rf crates/frontend/dist
