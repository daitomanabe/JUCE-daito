#!/usr/bin/env bash
# Build plugin-synth for both Apple Silicon and Intel and lipo the resulting
# binaries together into a Universal 2 bundle. Run from the rust/ directory.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script must be run on macOS." >&2
  exit 1
fi

CRATE="${1:-plugin-synth}"
PROFILE="release"

rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null

echo "==> Bundling for aarch64-apple-darwin..."
cargo xtask bundle "$CRATE" --release --target aarch64-apple-darwin

echo "==> Bundling for x86_64-apple-darwin..."
cargo xtask bundle "$CRATE" --release --target x86_64-apple-darwin

ARM_DIR="target/aarch64-apple-darwin/bundled"
INTEL_DIR="target/x86_64-apple-darwin/bundled"
OUT_DIR="target/universal-bundled"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

# Copy ARM bundles as the base, then lipo the Intel binaries into them.
cp -R "$ARM_DIR/." "$OUT_DIR/"

shopt -s nullglob
for bundle_path in "$OUT_DIR"/*.vst3 "$OUT_DIR"/*.clap "$OUT_DIR"/*.app; do
  name="$(basename "$bundle_path")"
  intel_path="$INTEL_DIR/$name"
  if [[ ! -e "$intel_path" ]]; then
    echo "  skip $name (no Intel counterpart)" >&2
    continue
  fi

  case "$name" in
    *.vst3|*.app)
      arm_bin=$(find "$bundle_path/Contents/MacOS" -type f -perm +111 | head -n1)
      intel_bin=$(find "$intel_path/Contents/MacOS" -type f -perm +111 | head -n1)
      ;;
    *.clap)
      # CLAP on macOS is a bundle with the binary at Contents/MacOS/<name>.
      arm_bin=$(find "$bundle_path/Contents/MacOS" -type f -perm +111 | head -n1)
      intel_bin=$(find "$intel_path/Contents/MacOS" -type f -perm +111 | head -n1)
      ;;
    *)
      continue
      ;;
  esac

  if [[ -z "${arm_bin:-}" || -z "${intel_bin:-}" ]]; then
    echo "  skip $name (could not locate Mach-O binaries)" >&2
    continue
  fi

  echo "  lipo $name"
  lipo -create "$arm_bin" "$intel_bin" -output "$arm_bin"
done

echo
echo "Universal bundles in $OUT_DIR:"
ls -1 "$OUT_DIR"
