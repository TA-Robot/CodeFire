#!/usr/bin/env bash
set -euo pipefail

prefix="/usr/local"
completion_shell=""
completion_dir=""

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
      echo "usage: install.sh [--prefix PATH] [--completion bash|zsh] [--completion-dir PATH]"
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
install -m 0755 "$script_dir/codefire" "$install_dir/codefire"
echo "installed: ${install_dir}/codefire"

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
