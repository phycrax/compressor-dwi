#!/bin/sh
set -ex

has_target() {
  rustup target list --installed | grep -q "$1"
}
ensure_target() {
  has_target "$1" || rustup target add "$1"
}

ensure_target thumbv6m-none-eabi

has_toolchain() {
  rustup toolchain list | grep -q "$1"
}
ensure_toolchain() {
  # The runner's default rustup profile is `minimal`, so request components explicitly.
  # `--allow-downgrade` picks the latest nightly that actually ships them.
  has_toolchain "$1" || rustup toolchain install "$1" --profile=minimal --component=rustfmt --allow-downgrade
}

has_component() {
  rustup component list --toolchain "$1" --installed | grep -q "$2"
}
ensure_component() {
  has_component "$1" "$2" || rustup component add --toolchain "$1" "$2"
}

ensure_toolchain nightly
# The toolchain may predate the check above (cached CI, local dev), so verify separately.
ensure_component nightly rustfmt

cargo_check() {
  cargo check --all "$@"
  cargo clippy --all "$@" -- --deny=warnings
}
cargo_test() {
  cargo_check --all-targets "$@"
  cargo test --all "$@"
}

cargo_test

cargo_check --target=thumbv6m-none-eabi
cargo_check --target=thumbv6m-none-eabi --no-default-features

cargo +nightly fmt --all -- --check

# Check docs.rs build
env RUSTDOCFLAGS='--cfg=docsrs --deny=warnings' cargo +nightly doc --all --no-deps --all-features
