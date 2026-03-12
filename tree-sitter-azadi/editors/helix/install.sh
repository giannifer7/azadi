#!/usr/bin/env bash
# editors/helix/install.sh
#
# Installs the azadi grammar and queries for Helix.
# Run once after cloning, and again after grammar changes.
#
# Usage:  bash editors/helix/install.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GRAMMAR_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"   # tree-sitter-azadi/
HELIX_RT="${XDG_CONFIG_HOME:-$HOME/.config}/helix/runtime"
QUERIES_DIR="$HELIX_RT/queries/azadi"

echo "Installing azadi grammar for Helix..."

# 1. Append language + grammar config if not already present
LANG_CONF="${XDG_CONFIG_HOME:-$HOME/.config}/helix/languages.toml"
if ! grep -q 'name = "azadi"' "$LANG_CONF" 2>/dev/null; then
  echo "" >> "$LANG_CONF"
  cat "$SCRIPT_DIR/languages.toml" >> "$LANG_CONF"
  echo "Appended azadi config to $LANG_CONF"
else
  echo "azadi already present in $LANG_CONF — skipping"
fi

# 2. Copy query files into Helix runtime
mkdir -p "$QUERIES_DIR"
cp "$GRAMMAR_DIR/queries/highlights.scm"  "$QUERIES_DIR/"
cp "$GRAMMAR_DIR/queries/injections.scm"  "$QUERIES_DIR/"
echo "Copied queries to $QUERIES_DIR"

# 3. Build the grammar
hx --grammar build
echo "Done.  Open a .azadi file in Helix to verify highlighting."
