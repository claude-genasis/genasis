#!/usr/bin/env sh
# Build genasis from source. Installs Rust if needed.
# Usage: curl ... | sh  OR  ./build.sh
set -eu

echo "=== genasis: build from source ==="

# Install Rust if not present
if ! command -v cargo >/dev/null 2>&1; then
    echo "[..] Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    . "$HOME/.cargo/env"
fi

echo "[..] Rust $(rustc --version)"

# Clone if not already in the repo
if [ ! -f "Cargo.toml" ] || ! grep -q 'genasis-cli' Cargo.toml 2>/dev/null; then
    echo "[..] Cloning genasis..."
    git clone https://github.com/claude-genasis/genasis.git
    cd genasis
fi

echo "[..] Building (release mode)..."
cargo build --release

BINARY="target/release/genasis"
if [ ! -f "$BINARY" ]; then
    echo "[FAIL] Build failed — binary not found."
    exit 1
fi

# Install
DEST="${HOME}/.local/bin"
mkdir -p "$DEST"
cp "$BINARY" "$DEST/genasis"
chmod +x "$DEST/genasis"

echo "[OK] Installed: $DEST/genasis"
echo "[OK] Version: $($DEST/genasis version 2>/dev/null || echo '(run genasis version to check)')"

# PATH check
case ":${PATH:-}:" in
    *":$DEST:"*) ;;
    *) echo "[WARN] $DEST is not in PATH. Add to your shell rc:"; echo "       export PATH=\"$DEST:\$PATH\"" ;;
esac

echo ""
echo "Next: genasis doctor    # verify environment"
echo "      genasis init      # set up agentic team"
