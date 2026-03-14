{
  description = "azadi — literate programming toolchain";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      pkgs    = nixpkgs.legacyPackages.x86_64-linux;
      version = "0.1.0";
      base    = "https://github.com/giannifer7/azadi/releases/download/v${version}";
      fetch   = filename: sha256: pkgs.fetchurl { url = "${base}/${filename}"; inherit sha256; };
    in {
      packages.x86_64-linux.default = pkgs.runCommand "azadi-${version}" {} ''
        mkdir -p $out/bin
        install -m755 ${fetch "azadi-musl"        "sha256-PLACEHOLDER"} $out/bin/azadi
        install -m755 ${fetch "azadi-macros-musl" "sha256-PLACEHOLDER"} $out/bin/azadi-macros
        install -m755 ${fetch "azadi-noweb-musl"  "sha256-PLACEHOLDER"} $out/bin/azadi-noweb
      '';
    };
}
