#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if ! command -v cargo >/dev/null 2>&1; then
  echo 'Rust/Cargo não encontrado. Instale o toolchain Rust no host local.' >&2
  exit 1
fi
export CARGO_TARGET_DIR="${TMPDIR:-/tmp}/colmeia-target"
echo 'Colmeia: http://127.0.0.1:7420/'
exec cargo run --release --bin colmeia -- web
