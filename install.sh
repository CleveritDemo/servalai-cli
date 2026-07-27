#!/bin/sh
# ServalAI CLI installer.
# Usage: curl -fsSL https://raw.githubusercontent.com/CleveritDemo/servalai-cli/main/install.sh | sh

set -eu

REPO="CleveritDemo/servalai-cli"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}/serval"
BIN="$HOME/.local/bin"
TOKEN_URL="https://cleverit-support.cleveritgroup.com"

# ── Detect platform ──────────────────────────────────────────────────────────

os="$(uname -s)"
arch="$(uname -m)"
case "$os-$arch" in
  Linux-x86_64)     target="x86_64-unknown-linux-musl"   ; friendly="Linux (x86_64)" ;;
  Linux-aarch64)    target="aarch64-unknown-linux-musl"  ; friendly="Linux (arm64)" ;;
  Darwin-x86_64)    target="x86_64-apple-darwin"          ; friendly="macOS (Intel)" ;;
  Darwin-arm64)     target="aarch64-apple-darwin"         ; friendly="macOS (Apple Silicon)" ;;
  *) echo ""; echo "  Unsupported platform: $os-$arch"; echo ""; exit 1 ;;
esac

# ── Fetch latest release ─────────────────────────────────────────────────────

echo ""
echo "  ░░░░░░░░░░  ServalAI CLI installer"
echo "  ░░▒▒░░▒▒░░  ─────────────────────"
echo "  ░▒▒▒▒▒▒▒▒░  Platform: $friendly"
echo "  ░░░▒▒▒▒░░░"
echo ""

echo "  Fetching latest release…"
tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep -m1 '"tag_name"' | cut -d'"' -f4)"
[ -z "$tag" ] && { echo "  Could not determine the latest release tag."; exit 1; }

asset="serval-$target.tar.gz"
url="https://github.com/$REPO/releases/download/$tag/$asset"

echo "  Downloading $tag ($target)…"

dest="$DATA/versions/$tag"
mkdir -p "$dest" "$BIN"
curl -fsSL "$url" | tar -xz -C "$dest"

# ── macOS Gatekeeper ─────────────────────────────────────────────────────────

if [ "$os" = "Darwin" ]; then
  xattr -dr com.apple.quarantine "$dest" 2>/dev/null || true
fi

ln -sfn "$dest" "$DATA/current"
ln -sfn "$DATA/current/serval" "$BIN/serval"

# ── PATH check ───────────────────────────────────────────────────────────────

case ":$PATH:" in
  *:"$BIN":*) path_ok=true ;;
  *)          path_ok=false ;;
esac

# ── Done ─────────────────────────────────────────────────────────────────────

echo ""
echo "  ✔   serval $tag installed."
echo ""

if ! $path_ok; then
  echo "  ⚠   $BIN is not on your PATH."
  echo "      Add this to your shell config (~/.zshrc or ~/.bashrc):"
  echo ""
  echo "          export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
  echo "      Then restart your terminal, or run:"
  echo ""
  echo "          export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
fi

echo "  ────────  Next steps  ────────"
echo ""
echo "  1.  Get your token at:"
echo "      $TOKEN_URL"
echo ""
echo "  2.  Authenticate:"
echo "      serval auth"
echo ""
echo "  3.  Start coding:"
echo "      serval"
echo ""

if command -v serval >/dev/null 2>&1; then
  serval status 2>/dev/null || true
fi