#!/usr/bin/env python3
"""Update sha256sums in PKGBUILD for a given release version."""

import argparse
import hashlib
import re
import urllib.request
from pathlib import Path

PKGBUILD = Path(__file__).parent / "PKGBUILD"
BASE_URL = "https://github.com/giannifer7/azadi/releases/download"


def sha256_of_url(url: str) -> str:
    print(f"Downloading {url} ...")
    with urllib.request.urlopen(url) as resp:
        digest = hashlib.sha256(resp.read()).hexdigest()
    print(f"sha256: {digest}")
    return digest


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="Release version, e.g. 0.1.0")
    args = parser.parse_args()

    version = args.version.lstrip("v")
    tarball_url = f"{BASE_URL}/v{version}/azadi-x86_64-linux.tar.gz"

    checksum = sha256_of_url(tarball_url)

    text = PKGBUILD.read_text()
    text = re.sub(r"^pkgver=.*$", f"pkgver={version}", text, flags=re.MULTILINE)
    text = re.sub(r"^sha256sums=\(.*?\)$", f"sha256sums=('{checksum}')",
                  text, flags=re.MULTILINE)
    PKGBUILD.write_text(text)
    print(f"Updated {PKGBUILD}")


if __name__ == "__main__":
    main()
