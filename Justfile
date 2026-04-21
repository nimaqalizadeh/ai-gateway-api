default: fmt lint test

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test *ARGS:
	cargo test {{ARGS}}

run:
	cargo run

docker-build:
	docker build -t ai-gateway .

