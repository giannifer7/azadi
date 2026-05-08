{
  description = "Bidirectional literate programming toolchain (noweb, macros, source tracing)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      lib     = nixpkgs.lib;
      version = "0.13.5";
      base    = "https://github.com/giannifer7/weaveback/releases/download/v${version}";

      # Pre-built musl binaries are x86_64-linux only.
      # They package the split CLI and supporting tools.
      #
      # The PyO3 extension is intentionally *not* exposed here as a pre-built
      # Nix package because it is Python-ABI- and platform-specific: for that
      # side we want wheels or a source build inside a dev shell, not a single
      # "universal musl" artifact.
      #
      # The devShell works on all common systems and includes the Python build
      # and lint tools needed for python/weaveback-agent and crates/weaveback-py.
      devSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachDevSystem = f: lib.genAttrs devSystems (s: f nixpkgs.legacyPackages.${s});

      linuxPkgs = nixpkgs.legacyPackages.x86_64-linux;

      releaseBin = { pname, sha256 }: linuxPkgs.stdenv.mkDerivation {
        inherit pname version;
        src        = linuxPkgs.fetchurl { url = "${base}/${pname}-musl"; inherit sha256; };
        dontUnpack = true;
        installPhase = "install -Dm755 $src $out/bin/${pname}";
      };

      cliBundle = linuxPkgs.stdenv.mkDerivation {
        pname      = "weaveback-cli";
        inherit version;
        src        = linuxPkgs.fetchurl { url = "${base}/weaveback-x86_64-linux.tar.gz"; sha256 = "sha256-kan8FPMdYo8a2M0xHBSN6PUBtlv8Da5f2Vnx9YvwhAA="; };
        dontUnpack = false;
        installPhase = ''
          install -Dm755 wb-tangle        $out/bin/wb-tangle
          install -Dm755 wb-query         $out/bin/wb-query
          install -Dm755 wb-serve         $out/bin/wb-serve
          install -Dm755 wb-mcp           $out/bin/wb-mcp
          install -Dm755 weaveback-macro  $out/bin/weaveback-macro
          install -Dm755 weaveback-tangle $out/bin/weaveback-tangle
          install -Dm755 weaveback-docgen $out/bin/weaveback-docgen
        '';
      };

    in {

      packages.x86_64-linux = {
        default          = cliBundle;
        weaveback-macro  = releaseBin { pname = "weaveback-macro";   sha256 = "sha256-HUCKBYuGJb/vkG9j4FK0KbHNAAtPXD80NOUSmLiqQNY="; };
        weaveback-tangle = releaseBin { pname = "weaveback-tangle";  sha256 = "sha256-XbpORmkYgzu1imFd6X0W8cibpaCMfNHAt3T7Mn9Zasg="; };
        weaveback-docgen = releaseBin { pname = "weaveback-docgen";  sha256 = "sha256-1m5SeA36XpqEvL9fEqGBOyJAP5ZDokJEp5IE+7lpPgk="; };
        wb-tangle        = releaseBin { pname = "wb-tangle";         sha256 = "sha256-LXb00t+9cA5B06e1auSq5OrWBFskG3bFacI1gDf/210="; };
        wb-query         = releaseBin { pname = "wb-query";          sha256 = "sha256-NahsMya0DEKbrtI5dTDKEu7DT7uJLjboN4eLRDs2pOg="; };
        wb-serve         = releaseBin { pname = "wb-serve";          sha256 = "sha256-WSnSboKK8XpYQNJsuw5DrXo8ZxD4qEoYStHES6//zVc="; };
        wb-mcp           = releaseBin { pname = "wb-mcp";            sha256 = "sha256-W5sWgh2UXrx94Kv2J/xphboDs/wRUC/A+Erjax6F1vU="; };
      };

      # Full documentation + development toolchain.
      # Usage: nix develop
      devShells = forEachDevSystem (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            just         # task runner
            plantuml     # UML diagrams via --plantuml-jar (brings JDK)
            nodejs       # TypeScript bundle for the serve UI
            python3      # packaging scripts and Python project runtime
            uv           # Python package / tool runner
            maturin      # PyO3 build frontend
            ruff         # Python formatter / linter
            mypy         # Python static typing
            pylint       # Python lint baseline
            git
          ];
          shellHook = ''
            echo ""
            echo "weaveback dev shell — available recipes:"
            echo "  just tangle     regenerate source files from .adoc"
            echo "  just docs       render HTML documentation"
            echo "  just serve      live-reload server with inline editor"
            echo "  just test       run all tests"
            echo "  just py-check   build + lint + test the Python agent bridge"
            if [ -f pyproject.toml ]; then
              echo "  syncing Python project with uv..."
              if ! uv sync --project . --all-groups; then
                echo "  warning: uv sync failed; continuing with the shell environment"
              fi
            fi
            echo ""
          '';
        };
      });
    };
}
