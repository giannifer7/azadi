#!/usr/bin/env python3
# editors/helix/install.py
#
# Installs the azadi grammar and queries for Helix.
# Run once after cloning, and again after grammar changes.
#
# Usage:  python3 editors/helix/install.py

import os
import shutil
import subprocess
import sys
from pathlib import Path

script_dir = Path(__file__).resolve().parent
grammar_dir = script_dir.parent.parent          # tree-sitter-azadi/

xdg = os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config")
helix_rt   = Path(xdg) / "helix" / "runtime"
queries_dir = helix_rt / "queries" / "azadi"
lang_conf   = Path(xdg) / "helix" / "languages.toml"

print("Installing azadi grammar for Helix...")

# 1. Append language + grammar config if not already present
snippet = (script_dir / "languages.toml").read_text()
if lang_conf.exists() and 'name = "azadi"' in lang_conf.read_text():
    print(f"azadi already present in {lang_conf} — skipping")
else:
    lang_conf.parent.mkdir(parents=True, exist_ok=True)
    with lang_conf.open("a") as f:
        f.write("\n" + snippet)
    print(f"Appended azadi config to {lang_conf}")

# 2. Copy query files into Helix runtime
queries_dir.mkdir(parents=True, exist_ok=True)
for name in ("highlights.scm", "injections.scm"):
    shutil.copy(grammar_dir / "queries" / name, queries_dir / name)
print(f"Copied queries to {queries_dir}")

# 3. Build the grammar
result = subprocess.run(["hx", "--grammar", "build"])
if result.returncode != 0:
    sys.exit(result.returncode)

print("Done.  Open a .azadi file in Helix to verify highlighting.")
