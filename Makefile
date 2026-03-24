.PHONY: lint test fmt check all run run-notify install install-notify

all: fmt lint test

run:
	cargo run

run-notify:
	cargo run --features notifications

install:
	cargo install --path .

install-notify:
	cargo install --path . --features notifications

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
