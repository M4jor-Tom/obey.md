{
  description = "GLTF Scene Orchestrator - LLM-driven multi-character animation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
    pkgsFor = nixpkgs.legacyPackages;
  in {
    apps = forAllSystems (system: {
      default = let
        python = pkgsFor.${system}.python313.withPackages (ps: with ps; [ openai ]);
        opencode = pkgsFor.${system}.opencode;
        wrapper = pkgsFor.${system}.writeShellScriptBin "obey" ''
          export PATH="${opencode}/bin:$PATH"
          cd "$PWD" && exec ${python}/bin/python -m src.orchestrator "$@"
        '';
      in {
        type = "app";
        program = "${wrapper}/bin/obey";
      };

      test = let
        python = pkgsFor.${system}.python313.withPackages (ps: with ps; [ openai pytest ]);
        testScript = pkgsFor.${system}.writeShellScriptBin "obey-test" ''
          exec ${python}/bin/python -m pytest src/ -v "$@"
        '';
      in {
        type = "app";
        program = "${testScript}/bin/obey-test";
      };
    });

    devShells = forAllSystems (system: {
      default = pkgsFor.${system}.mkShell {
        buildInputs = with pkgsFor.${system}; [
          python313
          python313Packages.pip
          python313Packages.pytest
          python313Packages.openai
          ffmpeg
          opencode
        ];
        shellHook = ''
          echo "GLTF Scene Orchestrator dev shell"
          echo "Run: python -m pytest src/ -v"
        '';
      };
    });
  };
}
