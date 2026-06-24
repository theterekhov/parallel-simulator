.PHONY: build run dev dev-front dev-back check clean

build:
	@echo "Building frontend..."
	cd crates/frontend && trunk build --release
	@echo "Building backend..."
	cargo build --release -p backend

run: build
	@echo "Starting server..."
	cargo run --release -p backend

dev:
	@echo "Запустите в двух терминалах"
	@echo "make dev-back (backend -> http://localhost:3000)"
	@echo "make dev-front (frontend -> http://localhost:8080)"

dev-back:
	cargo run -p backend

dev-front:
	cd crates/frontend && trunk serve

check:
	cargo check --workspace

clean:
	cargo clean && rm -rf crates/frontend/dist
