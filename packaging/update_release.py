#!/usr/bin/env python3
"""Generate PKGBUILD and flake.nix for a new release, then publish everywhere.

Typical usage:

  # Tag already pushed separately (just tag v0.x.y):
  python packaging/update_release.py 0.x.y

  # Let the script push the tag too:
  python packaging/update_release.py 0.x.y --tag
"""

import argparse
import base64
import hashlib
import json
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

PACKAGING   = Path(__file__).parent
REPO_ROOT   = PACKAGING.parent

MAINTAINER  = "Gianni Ferrarotti <gianni.ferrarotti@gmail.com>"
DESCRIPTION = "azadi — literate programming toolchain"
HOMEPAGE    = "https://github.com/giannifer7/azadi"
RELEASES    = f"{HOMEPAGE}/releases/download"

NEEDED_ASSETS = ["azadi-x86_64-linux.tar.gz", "azadi-musl"]


# ── hashing ────────────────────────────────────────────────────────────────────

def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_sri(data: bytes) -> str:
    return "sha256-" + base64.b64encode(hashlib.sha256(data).digest()).decode()


# ── file generators ────────────────────────────────────────────────────────────

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


# ── subprocess helpers ─────────────────────────────────────────────────────────

def run(args: list, cwd: Path) -> None:
    subprocess.run(args, cwd=cwd, check=True)


def gh(*args) -> subprocess.CompletedProcess:
    return subprocess.run(["gh", *args], check=True, capture_output=True, text=True)


# ── CI / release waiting ───────────────────────────────────────────────────────

def wait_for_release(version: str, timeout: int = 600, poll: int = 20) -> None:
    """Poll gh release view until all needed assets are present."""
    tag = f"v{version}"
    deadline = time.monotonic() + timeout
    print(f"Waiting for GitHub release {tag} assets", end="", flush=True)
    while time.monotonic() < deadline:
        r = subprocess.run(
            ["gh", "release", "view", tag, "--json", "assets"],
            capture_output=True, text=True,
        )
        if r.returncode == 0:
            names = {a["name"] for a in json.loads(r.stdout).get("assets", [])}
            if all(a in names for a in NEEDED_ASSETS):
                print(" ready.")
                return
        print(".", end="", flush=True)
        time.sleep(poll)
    raise SystemExit(f"\nTimed out after {timeout}s waiting for release assets.")


def download_assets(version: str, dest: Path) -> dict[str, bytes]:
    """Download release assets via gh and return their raw bytes."""
    tag = f"v{version}"
    patterns = [arg for name in NEEDED_ASSETS for arg in ("--pattern", name)]
    subprocess.run(
        ["gh", "release", "download", tag, *patterns, "--dir", str(dest), "--clobber"],
        check=True,
    )
    return {name: (dest / name).read_bytes() for name in NEEDED_ASSETS}


# ── main ───────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("version", help="Release version, e.g. 0.2.0")
    parser.add_argument("--tag", action="store_true",
                        help="Create and push the git tag before waiting for CI")
    parser.add_argument("--dry-run", action="store_true",
                        help="Write files but skip all git/AUR steps")
    args = parser.parse_args()

    version = args.version.lstrip("v")
    aur_dir = REPO_ROOT.parent / "aur-azadi-bin"

    if args.tag:
        print(f"Tagging v{version}...")
        run(["git", "tag", "-a", f"v{version}", "-m", f"v{version}"], cwd=REPO_ROOT)
        run(["git", "push", "origin", f"v{version}"], cwd=REPO_ROOT)

    wait_for_release(version)

    print("Downloading release artifacts via gh...")
    with tempfile.TemporaryDirectory() as tmp:
        assets = download_assets(version, Path(tmp))
        tarball   = assets["azadi-x86_64-linux.tar.gz"]
        azadi_bin = assets["azadi-musl"]

    (PACKAGING / "PKGBUILD").write_text(pkgbuild(version, sha256_hex(tarball)))
    print("  Written packaging/PKGBUILD")

    (REPO_ROOT / "flake.nix").write_text(flake(version, sha256_sri(azadi_bin)))
    print("  Written flake.nix")

    if args.dry_run:
        print("\nDry run — skipping git and AUR steps.")
        return

    print("\nCommitting azadi repo...")
    run(["git", "add", "flake.nix", "packaging/PKGBUILD"], cwd=REPO_ROOT)
    run(["git", "commit", "-m", f"chore: release v{version}"], cwd=REPO_ROOT)
    run(["git", "push", "origin", "main"], cwd=REPO_ROOT)

    print("\nUpdating AUR package...")
    shutil.copy(PACKAGING / "PKGBUILD", aur_dir / "PKGBUILD")
    srcinfo = subprocess.run(
        ["makepkg", "--printsrcinfo"],
        cwd=aur_dir, check=True, capture_output=True, text=True,
    ).stdout
    (aur_dir / ".SRCINFO").write_text(srcinfo)
    run(["git", "add", "PKGBUILD", ".SRCINFO"], cwd=aur_dir)
    run(["git", "commit", "-m", f"Release {version}"], cwd=aur_dir)
    run(["git", "push"], cwd=aur_dir)

    print(f"\nDone. Released v{version}.")


if __name__ == "__main__":
    main()
