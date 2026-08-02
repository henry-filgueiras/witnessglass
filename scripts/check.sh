#!/usr/bin/env bash
#
# The local gate for WitnessGlass. CI invokes this same script so local and CI
# semantics cannot drift apart.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "==> shell syntax"
for script in scripts/*.sh; do
    bash -n "$script"
done

echo "==> cargo fmt"
cargo fmt --all -- --check

echo "==> cargo clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "==> cargo test"
cargo test --workspace --all-targets --all-features

echo "==> scarp doctor"
scarp doctor

echo "==> all checks passed"
