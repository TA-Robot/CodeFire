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

# First-use walkthrough on the Rust CLI: init, open, extinguish, commit,
# clone, merge, and the file-backed remote flow.
cargo build --locked -p codefire-cli
demo_dir="$(mktemp -d)"
trap 'rm -rf "$demo_dir"' EXIT
CODEFIRE="$PWD/target/debug/codefire-rs" ./demo.sh "$demo_dir" >"$demo_dir.log" 2>&1 || {
  tail -n 20 "$demo_dir.log"
  echo "check: demo.sh failed on the Rust CLI (full log: $demo_dir.log)"
  exit 1
}
rm -f "$demo_dir.log"

echo "check: all gates passed"
