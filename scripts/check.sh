#!/usr/bin/env bash
# Single validation gate for the Rust product line.
# Usage: scripts/check.sh          (format, lint, build, all tests)
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v cargo >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --no-fail-fast
echo "check: all gates passed"
