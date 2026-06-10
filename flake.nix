{
  description = "GLTF Scene Orchestrator - LLM-driven multi-character animation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
    pkgsFor = nixpkgs.legacyPackages;

    gltf_to_png_script = system: let
      pkgs = pkgsFor.${system};
      render = pkgs.fetchurl {
        url = "https://raw.githubusercontent.com/M4jor-Tom/gltf_to_png.py/master/render.py";
        sha256 = "0l0912qp4bz51bc8qxk1a152prg66i5jmhij4qbf0i6bsc6jnnxi";
      };
    in pkgs.writeShellScriptBin "gltf_to_png" ''
      set -eu
      INPUT=""
      OUTPUT=""
      while [ $# -gt 0 ]; do
        case "$1" in
          -i) INPUT="$2"; shift 2 ;;
          -o) OUTPUT="$2"; shift 2 ;;
          *) echo "Unknown arg: $1"; exit 1 ;;
        esac
      done
      exec ${pkgs.blender}/bin/blender --background --python ${render} -- "$INPUT" "$OUTPUT"
    '';

    gltf_to_webm_script = system: let
      pkgs = pkgsFor.${system};
      render = pkgs.fetchurl {
        url = "https://raw.githubusercontent.com/M4jor-Tom/gltf_to_webm.py/master/render.py";
        sha256 = "1awj9xplsqkx3fk07qgk94vs3dgrq397w9683f52gr060n5s72sz";
      };
    in pkgs.writeShellScriptBin "gltf_to_webm" ''
      set -eu
      INPUT=""
      OUTPUT=""
      while [ $# -gt 0 ]; do
        case "$1" in
          -i) INPUT="$2"; shift 2 ;;
          -o) OUTPUT="$2"; shift 2 ;;
          *) echo "Unknown arg: $1"; exit 1 ;;
        esac
      done
      export PATH="${pkgs.ffmpeg}/bin:$PATH"
      exec ${pkgs.blender}/bin/blender --background --python ${render} -- "$INPUT" "$OUTPUT" 0 15 72 3.0
    '';
  in {
    apps = forAllSystems (system: let
      pkgs = pkgsFor.${system};
      gltf_to_png = gltf_to_png_script system;
      gltf_to_webm = gltf_to_webm_script system;
      python = pkgs.python313.withPackages (ps: with ps; [ openai loguru ]);
      opencode = pkgs.opencode;
    in {
      default = let
        wrapper = pkgs.writeShellScriptBin "obey" ''
          export PATH="${opencode}/bin:${gltf_to_png}/bin:${gltf_to_webm}/bin:$PATH"
          cd "$PWD" && exec ${python}/bin/python -m src.orchestrator "$@"
        '';
      in {
        type = "app";
        program = "${wrapper}/bin/obey";
      };

      test = let
        python = pkgs.python313.withPackages (ps: with ps; [ openai pytest loguru ]);
        testScript = pkgs.writeShellScriptBin "obey-test" ''
          exec ${python}/bin/python -m pytest src/ -v "$@"
        '';
      in {
        type = "app";
        program = "${testScript}/bin/obey-test";
      };
    });

    devShells = forAllSystems (system: let
      pkgs = pkgsFor.${system};
    in {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
          python313
          python313Packages.pip
          python313Packages.pytest
          python313Packages.openai
          python313Packages.loguru
          ffmpeg
          blender
          opencode
          (gltf_to_png_script system)
          (gltf_to_webm_script system)
        ];
        shellHook = ''
          echo "GLTF Scene Orchestrator dev shell"
          echo "Run: python -m pytest src/ -v"
        '';
      };
    });
  };
}
