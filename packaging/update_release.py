#!/usr/bin/env python3
"""Generate PKGBUILD and flake.nix for a new release."""

import argparse
import base64
import hashlib
import urllib.request
from pathlib import Path

PACKAGING   = Path(__file__).parent
REPO_ROOT   = PACKAGING.parent

MAINTAINER  = "Gianni Ferrarotti <gianni.ferrarotti@gmail.com>"
DESCRIPTION = "azadi — literate programming toolchain"
HOMEPAGE    = "https://github.com/giannifer7/azadi"
RELEASES    = f"{HOMEPAGE}/releases/download"


def fetch(url: str) -> bytes:
    print(f"  {url}")
    with urllib.request.urlopen(url) as r:
        return r.read()


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_sri(data: bytes) -> str:
    return "sha256-" + base64.b64encode(hashlib.sha256(data).digest()).decode()


def pkgbuild(version: str, tarball_sha256: str) -> str:
    source = f"{RELEASES}/v${{pkgver}}/azadi-x86_64-linux.tar.gz"
    return f"""\
# Maintainer: {MAINTAINER}
#
# AUR package for azadi — literate programming toolchain.
# Installs the azadi binary. The separate azadi-macros and azadi-noweb
# binaries are available in the GitHub release for advanced pipeline use.
#
# Regenerate after each release:
#   python packaging/update_release.py <version>

pkgname=azadi-bin
pkgver={version}
pkgrel=1
pkgdesc="{DESCRIPTION}"
url="{HOMEPAGE}"
license=('MIT' 'Apache-2.0')
arch=('x86_64')
provides=('azadi')
conflicts=('azadi' 'azadi-git')
depends=('gcc-libs' 'glibc')
options=('!debug')
source=("azadi-x86_64-linux.tar.gz::{source}")
sha256sums=('{tarball_sha256}')

package() {{
    install -Dm755 azadi -t "${{pkgdir}}/usr/bin"
}}
"""


def flake(version: str, sri_azadi: str) -> str:
    base = f"{RELEASES}/v${{version}}"
    return f"""\
{{
  description = "{DESCRIPTION}";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {{ self, nixpkgs }}:
    let
      pkgs    = nixpkgs.legacyPackages.x86_64-linux;
      version = "{version}";
      base    = "{base}";
    in {{
      packages.x86_64-linux.default = pkgs.stdenv.mkDerivation {{
        pname   = "azadi";
        inherit version;
        src     = pkgs.fetchurl {{ url = "${{base}}/azadi-musl"; sha256 = "{sri_azadi}"; }};
        dontUnpack   = true;
        installPhase = "install -Dm755 $src $out/bin/azadi";
      }};
    }};
}}
"""


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="Release version, e.g. 0.2.0")
    args = parser.parse_args()

    version = args.version.lstrip("v")
    base    = f"{RELEASES}/v{version}"

    print("Downloading release artifacts...")
    tarball = fetch(f"{base}/azadi-x86_64-linux.tar.gz")
    azadi   = fetch(f"{base}/azadi-musl")

    (PACKAGING / "PKGBUILD").write_text(
        pkgbuild(version, sha256_hex(tarball)))
    print("  Written packaging/PKGBUILD")

    (REPO_ROOT / "flake.nix").write_text(
        flake(version, sha256_sri(azadi)))
    print("  Written flake.nix")

    print(f"""
Done. Next steps:

  git add flake.nix packaging/PKGBUILD
  git commit -m "chore: release v{version}"
  git push origin main

  cp packaging/PKGBUILD ../aur-azadi-bin/
  cd ../aur-azadi-bin
  makepkg --printsrcinfo > .SRCINFO
  git add PKGBUILD .SRCINFO
  git commit -m "Release {version}"
  git push
""")


if __name__ == "__main__":
    main()
