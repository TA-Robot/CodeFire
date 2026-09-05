#!/usr/bin/env bash
# Source this file before local Rust setup/builds; it never downloads or installs.
# All state stays in this checkout. No shell profile or global toolchain changes.
codefire_rust_env() {
    local root
    root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd -P)" || return
    export CARGO_HOME="$root/build/local-rust/cargo"
    export RUSTUP_HOME="$root/build/local-rust/rustup"
    export CARGO_TARGET_DIR="$root/target"
    export TMPDIR="$root/build/local-rust/tmp"
    mkdir -p -- "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_TARGET_DIR" "$TMPDIR" || return
    case ":$PATH:" in
        *":$CARGO_HOME/bin:"*) ;;
        *) export PATH="$CARGO_HOME/bin:$PATH" ;;
    esac
}
if ! codefire_rust_env; then
    unset -f codefire_rust_env
    return 1
fi
unset -f codefire_rust_env
