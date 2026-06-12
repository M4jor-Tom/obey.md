{
  description = "GLTF Scene Orchestrator - LLM-driven multi-character animation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    graphify-src = {
      url = "github:safishamsi/graphify/v8";
      flake = false;
    };
    gltf-to-png = {
      url = "github:M4jor-Tom/gltf_to_png.py";
    };
    gltf-to-webm = {
      url = "github:M4jor-Tom/gltf_to_webm.py";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, graphify-src, gltf-to-png, gltf-to-webm }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      craneLib = crane.mkLib pkgs;

      graphify = pkgs.python313Packages.buildPythonPackage {
        pname = "graphifyy";
        version = "0.8.37";
        src = graphify-src;
        pyproject = true;
        postPatch = ''
          ${pkgs.gnused}/bin/sed -i \
            -e '/"tree-sitter-typescript",/d' \
            -e '/"tree-sitter-go",/d' \
            -e '/"tree-sitter-java",/d' \
            -e '/"tree-sitter-groovy",/d' \
            -e '/"tree-sitter-c",/d' \
            -e '/"tree-sitter-cpp",/d' \
            -e '/"tree-sitter-ruby",/d' \
            -e '/"tree-sitter-kotlin",/d' \
            -e '/"tree-sitter-scala",/d' \
            -e '/"tree-sitter-php",/d' \
            -e '/"tree-sitter-swift",/d' \
            -e '/"tree-sitter-lua",/d' \
            -e '/"tree-sitter-zig",/d' \
            -e '/"tree-sitter-powershell",/d' \
            -e '/"tree-sitter-elixir",/d' \
            -e '/"tree-sitter-objc",/d' \
            -e '/"tree-sitter-julia",/d' \
            -e '/"tree-sitter-verilog",/d' \
            -e '/"tree-sitter-fortran",/d' \
            pyproject.toml
          ${pkgs.gnused}/bin/sed -i \
            -e '/"tree-sitter-sql",/d' \
            -e '/"tree-sitter-hcl",/d' \
            pyproject.toml
        '';
        propagatedBuildInputs = with pkgs.python313Packages; [
          networkx numpy rapidfuzz tree-sitter
          tree-sitter-python tree-sitter-javascript
          tree-sitter-rust tree-sitter-c-sharp
          tree-sitter-bash tree-sitter-json
          setuptools
        ];
      };

      commonArgs = {
        pname = "obey";
        version = "0.1.0";
        src = craneLib.cleanCargoSource ./.;
      };

      obey = craneLib.buildPackage (commonArgs // {
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      });

      obey-clippy = craneLib.cargoClippy (commonArgs // {
        cargoArtifacts = obey;
        cargoClippyExtraArgs = "-- --deny warnings";
      });

      obey-doc = craneLib.cargoDoc (commonArgs // {
        cargoArtifacts = obey;
      });

      vizTools = pkgs.runCommand "viz-tools" {} ''
        mkdir -p $out/bin
        ln -s ${gltf-to-png.apps.${system}.default.program} $out/bin/gltf_to_png
        ln -s ${gltf-to-webm.apps.${system}.default.program} $out/bin/gltf_to_webm
      '';

    in {
      checks = {
        inherit obey-clippy;
        inherit obey-doc;
      };

      packages.default = obey;

      apps = {
        default = let
          opencode = pkgs.opencode;
          wrapper = pkgs.writeShellScriptBin "obey" ''
            export PATH="${vizTools}/bin:${opencode}/bin:$PATH"
            exec ${obey}/bin/obey "$@"
          '';
        in {
          type = "app";
          program = "${wrapper}/bin/obey";
        };

        test = let
          testScript = pkgs.writeShellScriptBin "obey-test" ''
            export PATH="${vizTools}/bin:$PATH"
            exec cargo test "$@"
          '';
        in {
          type = "app";
          program = "${testScript}/bin/obey-test";
        };
      };

      devShells.default = craneLib.devShell {
        inputsFrom = [ obey ];
        packages = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          opencode
          graphify
        ];
        shellHook = ''
          echo "GLTF Scene Orchestrator dev shell (Rust + Crane)"
          echo "Run: cargo build"
          echo "Run: cargo test"
        '';
      };
    });
}
