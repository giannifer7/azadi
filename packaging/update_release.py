#!/usr/bin/env python3
"""Update PKGBUILD and flake.nix checksums for a new release."""

import argparse
import base64
import hashlib
import re
import urllib.request
from pathlib import Path

PACKAGING = Path(__file__).parent
REPO_ROOT  = PACKAGING.parent
PKGBUILD   = PACKAGING / "PKGBUILD"
FLAKE      = REPO_ROOT  / "flake.nix"
BASE_URL   = "https://github.com/giannifer7/azadi/releases/download"


def fetch(url: str) -> bytes:
    print(f"  {url}")
    with urllib.request.urlopen(url) as r:
        return r.read()


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_sri(data: bytes) -> str:
    return "sha256-" + base64.b64encode(hashlib.sha256(data).digest()).decode()


def patch(path: Path, subs: list[tuple[str, str]]) -> None:
    text = path.read_text()
    for pattern, replacement in subs:
        text = re.sub(pattern, replacement, text, flags=re.MULTILINE)
    path.write_text(text)
    print(f"  Updated {path.relative_to(REPO_ROOT)}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="Release version, e.g. 0.2.0")
    args = parser.parse_args()

    version = args.version.lstrip("v")
    base    = f"{BASE_URL}/v{version}"

    print("Downloading release artifacts...")
    tarball      = fetch(f"{base}/azadi-x86_64-linux.tar.gz")
    azadi        = fetch(f"{base}/azadi-musl")
    azadi_macros = fetch(f"{base}/azadi-macros-musl")
    azadi_noweb  = fetch(f"{base}/azadi-noweb-musl")

    print("\nPatching PKGBUILD...")
    patch(PKGBUILD, [
        (r"^pkgver=.*$",         f"pkgver={version}"),
        (r"^sha256sums=\(.*?\)$", f"sha256sums=('{sha256_hex(tarball)}')"),
    ])

    print("Patching flake.nix...")
    def flake_sub(filename: str, sri: str) -> tuple[str, str]:
        return (
            rf'(fetch\s+"{re.escape(filename)}"\s+)"[^"]*"',
            rf'\1"{sri}"',
        )
    patch(FLAKE, [
        (r'(version\s*=\s*)"[^"]*"', rf'\1"{version}"'),
        flake_sub("azadi-musl",        sha256_sri(azadi)),
        flake_sub("azadi-macros-musl", sha256_sri(azadi_macros)),
        flake_sub("azadi-noweb-musl",  sha256_sri(azadi_noweb)),
    ])

    print(f"""
Done. Next steps:

  git add flake.nix packaging/PKGBUILD
  git commit -m "chore: release v{version}"
  git push origin main

  cp packaging/PKGBUILD ~/aur-azadi-bin/
  cd ~/aur-azadi-bin
  makepkg --printsrcinfo > .SRCINFO
  git add PKGBUILD .SRCINFO
  git commit -m "Release {version}"
  git push
""")


if __name__ == "__main__":
    main()
