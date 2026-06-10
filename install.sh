#!/usr/bin/env bash
set -euo pipefail

prefix="/usr/local"
completion_shell=""
completion_dir=""
rust_binary="${CODEFIRE_RUST_BINARY:-}"
dry_run=0

die() {
  echo "error: $*" >&2
  exit 2
}

validate_prefix() {
  if [[ -z "$prefix" ]]; then
    die "--prefix requires a value"
  fi
  if [[ "$prefix" != /* ]]; then
    die "--prefix must be an absolute path: $prefix"
  fi
  if [[ "$prefix" == "/" ]]; then
    die "--prefix / is not allowed"
  fi
  if [[ -e "$prefix" && ! -d "$prefix" ]]; then
    die "--prefix exists but is not a directory: $prefix"
  fi
  if [[ -L "$prefix" ]]; then
    die "--prefix must not be a symlink: $prefix"
  fi
  if [[ -L "$prefix/bin" ]]; then
    die "install bin directory must not be a symlink: $prefix/bin"
  fi
}

verify_codefire_binary() {
  local binary="$1"
  local help_output version_output
  if [[ ! -x "$binary" ]]; then
    die "Rust codefire binary is not executable: $binary"
  fi
  help_output="$("$binary" --help 2>&1)" || die "Rust codefire binary failed --help smoke: $binary"
  if [[ "$help_output" != *"usage: codefire <command> [args]"* ]]; then
    die "Rust codefire binary --help output does not look like CodeFire: $binary"
  fi
  version_output="$("$binary" --version 2>&1)" || die "Rust codefire binary failed --version smoke: $binary"
  if [[ "$version_output" != codefire\ foundation\ * ]]; then
    die "Rust codefire binary --version output does not look like CodeFire: $binary"
  fi
}

report_active_codefire() {
  local installed="$1"
  local active
  active="$(command -v codefire 2>/dev/null || true)"
  if [[ -z "$active" ]]; then
    echo "warning: no active codefire found on PATH; use ${installed} or add ${install_dir} to PATH" >&2
    return 0
  fi
  echo "active codefire: ${active}"
  if [[ "$active" != "$installed" ]]; then
    echo "warning: PATH resolves codefire to ${active}, not installed target ${installed}" >&2
    echo "warning: update PATH order, choose a prefix earlier on PATH, or invoke ${installed} directly" >&2
    return 0
  fi
  verify_codefire_binary "$active"
  echo "active codefire verified: ${active}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      prefix="${2:-}"
      [[ -n "$prefix" ]] || die "--prefix requires a value"
      shift 2
      ;;
    --prefix=*)
      prefix="${1#--prefix=}"
      shift
      ;;
    --binary)
      rust_binary="${2:-}"
      [[ -n "$rust_binary" ]] || die "--binary requires a value"
      shift 2
      ;;
    --binary=*)
      rust_binary="${1#--binary=}"
      [[ -n "$rust_binary" ]] || die "--binary requires a value"
      shift
      ;;
    --completion)
      completion_shell="${2:-}"
      if [[ "$completion_shell" != "bash" && "$completion_shell" != "zsh" ]]; then
        die "--completion requires 'bash' or 'zsh'"
      fi
      shift 2
      ;;
    --completion=*)
      completion_shell="${1#--completion=}"
      if [[ "$completion_shell" != "bash" && "$completion_shell" != "zsh" ]]; then
        die "--completion requires 'bash' or 'zsh'"
      fi
      shift
      ;;
    --completion-dir)
      completion_dir="${2:-}"
      [[ -n "$completion_dir" ]] || die "--completion-dir requires a value"
      shift 2
      ;;
    --completion-dir=*)
      completion_dir="${1#--completion-dir=}"
      shift
      ;;
    --dry-run)
      dry_run=1
      shift
      ;;
    -h|--help)
      echo "usage: install.sh [--prefix PATH] [--binary PATH] [--completion bash|zsh] [--completion-dir PATH] [--dry-run]"
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
install_dir="${prefix}/bin"
validate_prefix

if [[ -n "$completion_shell" && -z "$completion_dir" ]]; then
  if [[ "$completion_shell" == "bash" ]]; then
    completion_dir="${prefix}/share/bash-completion/completions"
  else
    completion_dir="${prefix}/share/zsh/site-functions"
  fi
fi

if [[ -n "$completion_dir" ]]; then
  if [[ "$completion_dir" != /* ]]; then
    die "--completion-dir must be an absolute path: $completion_dir"
  fi
  if [[ -e "$completion_dir" && ! -d "$completion_dir" ]]; then
    die "--completion-dir exists but is not a directory: $completion_dir"
  fi
  if [[ -L "$completion_dir" ]]; then
    die "--completion-dir must not be a symlink: $completion_dir"
  fi
fi

if [[ "$dry_run" -eq 1 ]]; then
  planned_binary="$rust_binary"
  if [[ -z "$planned_binary" ]]; then
    if [[ -x "$script_dir/target/release/codefire-rs" ]]; then
      planned_binary="$script_dir/target/release/codefire-rs"
    else
      planned_binary="<build with cargo build --release -p codefire-cli --bin codefire-rs>"
    fi
  fi
  echo "CodeFire install plan"
  echo "dry_run: true"
  echo "prefix: $prefix"
  echo "install_dir: $install_dir"
  echo "rust_binary: $planned_binary"
  echo "python_fallback: $script_dir/codefire"
  echo "install_codefire: $install_dir/codefire"
  echo "install_codefire_py: $install_dir/codefire-py"
  if [[ -n "$completion_shell" ]]; then
    echo "completion_shell: $completion_shell"
    if [[ "$completion_shell" == "bash" ]]; then
      echo "completion_file: $completion_dir/codefire"
    else
      echo "completion_file: $completion_dir/_codefire"
    fi
  else
    echo "completion_shell: none"
  fi
  exit 0
fi

mkdir -p "$install_dir"

if [[ -z "$rust_binary" ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    die "cargo is required to build the Rust codefire binary; pass --binary PATH to use a prebuilt binary"
  fi
  (cd "$script_dir" && cargo build --release -p codefire-cli --bin codefire-rs)
  rust_binary="$script_dir/target/release/codefire-rs"
fi

verify_codefire_binary "$rust_binary"
if [[ ! -f "$script_dir/codefire" ]]; then
  die "Python fallback script is missing: $script_dir/codefire"
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/codefire-install.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT
completion_tmp=""
completion_target=""
if [[ -n "$completion_shell" ]]; then
  mkdir -p "$completion_dir"
  if [[ "$completion_shell" == "bash" ]]; then
    completion_tmp="$tmp_dir/codefire"
    completion_target="$completion_dir/codefire"
    "$rust_binary" completion bash > "$completion_tmp"
  else
    completion_tmp="$tmp_dir/_codefire"
    completion_target="$completion_dir/_codefire"
    "$rust_binary" completion zsh > "$completion_tmp"
  fi
fi

install -m 0755 "$rust_binary" "$install_dir/codefire"
install -m 0755 "$script_dir/codefire" "$install_dir/codefire-py"
verify_codefire_binary "$install_dir/codefire"
echo "installed Rust CLI: ${install_dir}/codefire"
echo "installed Python fallback: ${install_dir}/codefire-py"

if [[ -n "$completion_tmp" ]]; then
  install -m 0644 "$completion_tmp" "$completion_target"
  echo "installed: ${completion_target}"
fi

report_active_codefire "${install_dir}/codefire"
