#!/usr/bin/env bash
set -euo pipefail

# Builds an aarch64 Debian package for the `tuwunel` binary using either
# `cross` (recommended) or a local cross toolchain. The script will:
#  - build the aarch64 release binary
#  - copy it to `target/release/tuwunel` (cargo-deb expects that path)
#  - run `cargo deb` to produce a .deb
#
# Usage: ./scripts/build-deb-aarch64.sh

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

TARGET=aarch64-unknown-linux-gnu
CRATE=tuwunel

out_dir="out"
mkdir -p "$out_dir"

echo "Building Debian package for target: $TARGET"

if command -v cross >/dev/null 2>&1; then
  echo "Using 'cross' to build (Docker/qemu powered)",\
       " - this usually works without needing a native cross toolchain."
  cross build --release --target "$TARGET" -p "$CRATE"
else
  echo "'cross' not found. Falling back to direct 'cargo' cross-build."
  echo "Make sure you have rustup target '$TARGET' installed and an aarch64 linker (gcc-aarch64-linux-gnu) available."
  if ! rustup target list --installed | grep -q "^$TARGET$"; then
    echo "Adding rust target: $TARGET"
    rustup target add "$TARGET"
  fi

  # Try to build; this can fail if no cross linker is installed
  cargo build --release --target "$TARGET" -p "$CRATE"
fi

BIN_SRC="target/$TARGET/release/$CRATE"
if [ ! -f "$BIN_SRC" ]; then
  echo "Built binary not found at $BIN_SRC" >&2
  echo "If you used 'cargo' directly you may need to install an aarch64 linker or use 'cross'." >&2
  exit 1
fi

mkdir -p target/release
cp -v "$BIN_SRC" target/release/$CRATE

if ! command -v cargo-deb >/dev/null 2>&1; then
  echo "cargo-deb not found, installing it now (cargo install cargo-deb)"
  cargo install cargo-deb
fi

echo "Running cargo deb (no build) for target: $TARGET"
# Use --no-build because we've already built the binary for the given target.
cargo deb --no-build --target "$TARGET" -p "$CRATE"

# cargo-deb writes the .deb into target/debian/ or target/debian/<arch>/
debfile=$(ls -1 target/debian/*.deb 2>/dev/null || true)
if [ -z "$debfile" ]; then
  # try recursive search
  debfile=$(find target -maxdepth 3 -type f -name "*.deb" | head -n1 || true)
fi

if [ -z "$debfile" ]; then
  echo "Failed to locate the generated .deb file." >&2
  exit 1
fi

cp -v "$debfile" "$out_dir/"

echo "Created: $out_dir/$(basename "$debfile")"
echo "Done."
