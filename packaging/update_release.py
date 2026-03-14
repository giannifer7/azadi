#!/usr/bin/env python3
"""Generate PKGBUILD and flake.nix for a new release."""

import argparse
import base64
import hashlib
import urllib.request
from pathlib import Path

PACKAGING = Path(__file__).parent
REPO_ROOT  = PACKAGING.parent
BASE_URL   = "https://github.com/giannifer7/azadi/releases/download"


def fetch(url: str) -> bytes:
    print(f"  {url}")
    with urllib.request.urlopen(url) as r:
        return r.read()


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_sri(data: bytes) -> str:
    return "sha256-" + base64.b64encode(hashlib.sha256(data).digest()).decode()


def pkgbuild(version: str, tarball_sha256: str) -> str:
    return f"""\
# Maintainer: Gianni Ferrarotti <gianni.ferrarotti@gmail.com>
#
# AUR package for azadi — literate programming toolchain.
# Installs three binaries: azadi, azadi-macros, azadi-noweb.
#
# Regenerate after each release:
#   python packaging/update_release.py <version>

pkgname=azadi-bin
pkgver={version}
pkgrel=1
pkgdesc="azadi — literate programming toolchain"
url="https://github.com/giannifer7/azadi"
license=('MIT' 'Apache-2.0')
arch=('x86_64')
provides=('azadi')
conflicts=('azadi' 'azadi-git')
depends=('gcc-libs' 'glibc')
options=('!debug')
source=("azadi-x86_64-linux.tar.gz::https://github.com/giannifer7/azadi/releases/download/v${{pkgver}}/azadi-x86_64-linux.tar.gz")
sha256sums=('{tarball_sha256}')

package() {{
    install -Dm755 azadi        -t "${{pkgdir}}/usr/bin"
    install -Dm755 azadi-macros -t "${{pkgdir}}/usr/bin"
    install -Dm755 azadi-noweb  -t "${{pkgdir}}/usr/bin"
}}
"""


def flake(version: str, sri_azadi: str, sri_macros: str, sri_noweb: str) -> str:
    return f"""\
{{
  description = "azadi — literate programming toolchain";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {{ self, nixpkgs }}:
    let
      pkgs    = nixpkgs.legacyPackages.x86_64-linux;
      version = "{version}";
      base    = "https://github.com/giannifer7/azadi/releases/download/v${{version}}";
      fetch   = filename: sha256: pkgs.fetchurl {{ url = "${{base}}/${{filename}}"; inherit sha256; }};
    in {{
      packages.x86_64-linux.default = pkgs.runCommand "azadi-${{version}}" {{}} ''
        mkdir -p $out/bin
        install -m755 ${{fetch "azadi-musl"        "{sri_azadi}"}} $out/bin/azadi
        install -m755 ${{fetch "azadi-macros-musl" "{sri_macros}"}} $out/bin/azadi-macros
        install -m755 ${{fetch "azadi-noweb-musl"  "{sri_noweb}"}} $out/bin/azadi-noweb
      '';
    }};
}}
"""


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

    out_pkgbuild = pkgbuild(version, sha256_hex(tarball))
    out_flake    = flake(version, sha256_sri(azadi), sha256_sri(azadi_macros), sha256_sri(azadi_noweb))

    (PACKAGING / "PKGBUILD").write_text(out_pkgbuild)
    print(f"  Written packaging/PKGBUILD")

    (REPO_ROOT / "flake.nix").write_text(out_flake)
    print(f"  Written flake.nix")

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
