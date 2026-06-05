#!/usr/bin/env bash
set -euo pipefail

prefix="/usr/local"
completion_shell=""
completion_dir=""
rust_binary="${CODEFIRE_RUST_BINARY:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      prefix="${2:-}"
      if [[ -z "$prefix" ]]; then
        echo "error: --prefix requires a value" >&2
        exit 2
      fi
      shift 2
      ;;
    --prefix=*)
      prefix="${1#--prefix=}"
      shift
      ;;
    --binary)
      rust_binary="${2:-}"
      if [[ -z "$rust_binary" ]]; then
        echo "error: --binary requires a value" >&2
        exit 2
      fi
      shift 2
      ;;
    --binary=*)
      rust_binary="${1#--binary=}"
      if [[ -z "$rust_binary" ]]; then
        echo "error: --binary requires a value" >&2
        exit 2
      fi
      shift
      ;;
    --completion)
      completion_shell="${2:-}"
      if [[ "$completion_shell" != "bash" && "$completion_shell" != "zsh" ]]; then
        echo "error: --completion requires 'bash' or 'zsh'" >&2
        exit 2
      fi
      shift 2
      ;;
    --completion=*)
      completion_shell="${1#--completion=}"
      if [[ "$completion_shell" != "bash" && "$completion_shell" != "zsh" ]]; then
        echo "error: --completion requires 'bash' or 'zsh'" >&2
        exit 2
      fi
      shift
      ;;
    --completion-dir)
      completion_dir="${2:-}"
      if [[ -z "$completion_dir" ]]; then
        echo "error: --completion-dir requires a value" >&2
        exit 2
      fi
      shift 2
      ;;
    --completion-dir=*)
      completion_dir="${1#--completion-dir=}"
      shift
      ;;
    -h|--help)
      echo "usage: install.sh [--prefix PATH] [--binary PATH] [--completion bash|zsh] [--completion-dir PATH]"
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      exit 2
      ;;
  esac
done

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
install_dir="${prefix}/bin"
mkdir -p "$install_dir"

if [[ -z "$rust_binary" ]]; then
  if [[ -x "$script_dir/target/release/codefire-rs" ]]; then
    rust_binary="$script_dir/target/release/codefire-rs"
  else
    if ! command -v cargo >/dev/null 2>&1; then
      echo "error: cargo is required to build the Rust codefire binary; pass --binary PATH to use a prebuilt binary" >&2
      exit 2
    fi
    (cd "$script_dir" && cargo build --release -p codefire-cli --bin codefire-rs)
    rust_binary="$script_dir/target/release/codefire-rs"
  fi
fi

if [[ ! -x "$rust_binary" ]]; then
  echo "error: Rust codefire binary is not executable: $rust_binary" >&2
  exit 2
fi
if [[ ! -f "$script_dir/codefire" ]]; then
  echo "error: Python fallback script is missing: $script_dir/codefire" >&2
  exit 2
fi

install -m 0755 "$rust_binary" "$install_dir/codefire"
install -m 0755 "$script_dir/codefire" "$install_dir/codefire-py"
echo "installed Rust CLI: ${install_dir}/codefire"
echo "installed Python fallback: ${install_dir}/codefire-py"

if [[ -n "$completion_shell" ]]; then
  if [[ -z "$completion_dir" ]]; then
    if [[ "$completion_shell" == "bash" ]]; then
      completion_dir="${prefix}/share/bash-completion/completions"
    else
      completion_dir="${prefix}/share/zsh/site-functions"
    fi
  fi
  mkdir -p "$completion_dir"
  if [[ "$completion_shell" == "bash" ]]; then
    "$install_dir/codefire" completion bash > "$completion_dir/codefire"
    echo "installed: ${completion_dir}/codefire"
  else
    "$install_dir/codefire" completion zsh > "$completion_dir/_codefire"
    echo "installed: ${completion_dir}/_codefire"
  fi
fi
