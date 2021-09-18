all: install-tools build check

install-tools:
	rustup component add clippy rustfmt

build:
	cargo build

check: check-check check-test check-format check-lint

check-check:
	cargo check

check-format:
	cargo fmt --all -- --check

check-lint:
	cargo clippy

check-test:
	cargo test
