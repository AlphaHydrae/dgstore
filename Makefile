all: install-tools build check

install-tools:
	rustup component add llvm-tools-preview && \
	cargo install grcov && \
	{ asdf where rust &>/dev/null && asdf reshim rust || echo "Not using asdf"; }

build:
	export RUSTC_BOOTSTRAP=1 && \
	export RUSTFLAGS="-Zinstrument-coverage" && \
	cargo build

check: check-test check-coverage check-format

check-test:
	export RUSTC_BOOTSTRAP=1 && \
	export RUSTFLAGS="-Zinstrument-coverage" && \
	export LLVM_PROFILE_FILE="target/debug/dgstore-%p-%m.profraw" && \
	cargo test

check-coverage: check-coverage-html check-coverage-lcov clean-coverage

check-coverage-html:
	export LLVM_PROFILE_FILE="target/debug/dgstore-%p-%m.profraw" && \
	grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./coverage/

check-coverage-lcov:
	export LLVM_PROFILE_FILE="target/debug/dgstore-%p-%m.profraw" && \
	grcov . -s . --binary-path ./target/debug/ -t lcov --branch --ignore-not-existing -o ./coverage/rust.lcov

clean-coverage:
	rm -f ./target/debug/*.profraw

check-format:
	cargo fmt -- --check
