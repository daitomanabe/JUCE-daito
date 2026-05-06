#!/usr/bin/env bash
# Install the bundled plug-ins into the user's Audio Plug-Ins folder. Run
# 'cargo xtask bundle plugin-synth --release' (or mac-bundle-universal.sh)
# first.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script must be run on macOS." >&2
  exit 1
fi

SOURCE="${1:-target/bundled}"
if [[ ! -d "$SOURCE" ]]; then
  echo "Source directory '$SOURCE' does not exist." >&2
  exit 1
fi

VST3_DEST="$HOME/Library/Audio/Plug-Ins/VST3"
CLAP_DEST="$HOME/Library/Audio/Plug-Ins/CLAP"
mkdir -p "$VST3_DEST" "$CLAP_DEST"

shopt -s nullglob
for bundle in "$SOURCE"/*.vst3; do
  name="$(basename "$bundle")"
  echo "==> $name -> $VST3_DEST/"
  rm -rf "$VST3_DEST/$name"
  cp -R "$bundle" "$VST3_DEST/"
done

for bundle in "$SOURCE"/*.clap; do
  name="$(basename "$bundle")"
  echo "==> $name -> $CLAP_DEST/"
  rm -rf "$CLAP_DEST/$name"
  cp -R "$bundle" "$CLAP_DEST/"
done

echo
echo "Done. Restart your DAW or rescan plug-ins to pick them up."
