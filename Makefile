.PHONY: fmt fmt-check check test clippy doc build ci

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets

test:
	cargo test --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

doc:
	cargo doc --workspace --all-features --no-deps

build:
	cargo build --workspace --all-targets

ci: fmt-check check test clippy doc

# Windows users can run the individual cargo commands directly; GNU Make is optional.
