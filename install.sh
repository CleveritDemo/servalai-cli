#!/bin/sh
# ServalAI CLI installer. Usage: curl -fsSL <url>/install.sh | sh
set -eu

REPO="CleveritDemo/servalai-cli"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}/serval"
BIN="$HOME/.local/bin"

os="$(uname -s)"; arch="$(uname -m)"
case "$os-$arch" in
  Linux-x86_64)  target="x86_64-unknown-linux-musl" ;;
  Linux-aarch64) target="aarch64-unknown-linux-musl" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  Darwin-arm64)  target="aarch64-apple-darwin" ;;
  *) echo "unsupported platform: $os-$arch" >&2; exit 1 ;;
esac

tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep -m1 '"tag_name"' | cut -d'"' -f4)"
asset="serval-$target.tar.gz"
url="https://github.com/$REPO/releases/download/$tag/$asset"

echo "Installing serval $tag for $target..." >&2
dest="$DATA/versions/$tag"
mkdir -p "$dest" "$BIN"
curl -fsSL "$url" | tar -xz -C "$dest"

# macOS: clear quarantine so the unsigned binaries run.
if [ "$os" = "Darwin" ]; then
  xattr -dr com.apple.quarantine "$dest" 2>/dev/null || true
fi

ln -sfn "$dest" "$DATA/current"
ln -sfn "$DATA/current/serval" "$BIN/serval"

echo "Installed. Ensure $BIN is on your PATH, then run: serval auth" >&2
