#!/bin/sh
# F49 — Gera um pacote .deb mínimo do Colmeia (sem crates extras no alvo).
# Uso: sh packaging/debian/build.sh [caminho-do-binario]
# Requer: dpkg-deb (pacote dpkg) e um binário release em target/release/colmeia.
set -eu

BIN="${1:-target/release/colmeia}"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
[ -n "$VERSION" ] || { echo "versão não encontrada no Cargo.toml"; exit 1; }
[ -f "$BIN" ] || { echo "binário não encontrado: $BIN (rode 'cargo build --release')"; exit 1; }

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

mkdir -p "$STAGE/usr/local/bin"
mkdir -p "$STAGE/DEBIAN"
cp "$BIN" "$STAGE/usr/local/bin/colmeia"
chmod 755 "$STAGE/usr/local/bin/colmeia"

cat > "$STAGE/DEBIAN/control" <<EOF
Package: colmeia
Version: $VERSION
Section: devel
Priority: optional
Architecture: amd64
Depends: git
Maintainer: Colmeia App Team
Description: Gestor Git de frotas, TUI-first, determinístico em hardware mínimo
EOF

OUT="colmeia_${VERSION}_amd64.deb"
dpkg-deb --build "$STAGE" "$OUT"
echo "pacote gerado: $OUT"
