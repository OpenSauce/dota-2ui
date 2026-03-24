.PHONY: lint test fmt check all

all: fmt lint test

fmt:
	cargo fmt --all

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets --features notifications -- -D warnings

test:
	cargo test --all-targets

check:
	cargo check --all-targets
	cargo check --all-targets --features notifications
