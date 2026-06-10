{
  description = "GLTF Scene Orchestrator - LLM-driven multi-character animation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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

  outputs = { self, nixpkgs, graphify-src, gltf-to-png, gltf-to-webm }: let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
    pkgsFor = nixpkgs.legacyPackages;
    graphifyFor = system: pkgsFor.${system}.python313Packages.buildPythonPackage {
      pname = "graphifyy";
      version = "0.8.37";
      src = graphify-src;
      pyproject = true;
      postPatch = ''
        # Remove tree-sitter grammars not in nixpkgs — graphify still works,
        # it just won't do AST extraction for those languages.
        ${pkgsFor.${system}.gnused}/bin/sed -i \
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
        ${pkgsFor.${system}.gnused}/bin/sed -i \
          -e '/"tree-sitter-sql",/d' \
          -e '/"tree-sitter-hcl",/d' \
          pyproject.toml
      '';
      propagatedBuildInputs = with pkgsFor.${system}.python313Packages; [
        networkx numpy rapidfuzz tree-sitter
        tree-sitter-python tree-sitter-javascript
        tree-sitter-rust tree-sitter-c-sharp
        tree-sitter-bash tree-sitter-json
        setuptools
      ];
    };
  in {
    apps = forAllSystems (system: let
      graphify = graphifyFor system;
      vizTools = pkgsFor.${system}.runCommand "viz-tools" {} ''
        mkdir -p $out/bin
        ln -s ${gltf-to-png.apps.${system}.default.program} $out/bin/gltf_to_png
        ln -s ${gltf-to-webm.apps.${system}.default.program} $out/bin/gltf_to_webm
      '';
    in {
      default = let
        python = pkgsFor.${system}.python313.withPackages (ps: with ps; [ openai loguru graphify ]);
        opencode = pkgsFor.${system}.opencode;
        wrapper = pkgsFor.${system}.writeShellScriptBin "obey" ''
          export PATH="${vizTools}/bin:${opencode}/bin:$PATH"
          cd "$PWD" && exec ${python}/bin/python -m src.orchestrator "$@"
        '';
      in {
        type = "app";
        program = "${wrapper}/bin/obey";
      };

      test = let
        python = pkgsFor.${system}.python313.withPackages (ps: with ps; [ openai pytest loguru graphify ]);
        testScript = pkgsFor.${system}.writeShellScriptBin "obey-test" ''
          export PATH="${vizTools}/bin:$PATH"
          exec ${python}/bin/python -m pytest src/ -v "$@"
        '';
      in {
        type = "app";
        program = "${testScript}/bin/obey-test";
      };
    });

    devShells = forAllSystems (system: let
      graphify = graphifyFor system;
    in {
      default = pkgsFor.${system}.mkShell {
        buildInputs = with pkgsFor.${system}; [
          python313
          python313Packages.pip
          python313Packages.pytest
          python313Packages.openai
          python313Packages.loguru
          graphify
          opencode
        ];
        shellHook = ''
          echo "GLTF Scene Orchestrator dev shell"
          echo "Run: python -m pytest src/ -v"
          # Install missing tree-sitter grammars not in nixpkgs
          pip install --quiet \
            tree-sitter-typescript tree-sitter-go tree-sitter-java \
            tree-sitter-groovy tree-sitter-c tree-sitter-cpp \
            tree-sitter-ruby tree-sitter-kotlin tree-sitter-scala \
            tree-sitter-php tree-sitter-swift tree-sitter-lua \
            tree-sitter-zig tree-sitter-powershell tree-sitter-elixir \
            tree-sitter-objc tree-sitter-julia tree-sitter-verilog \
            tree-sitter-fortran 2>/dev/null || true
        '';
      };
    });
  };
}
