{
  description = "DecDev — a catalog of declaratively-specified game-development capabilities";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      # CLI and infra (crates/core, cli/, and any future api/mcp crate) are
      # Rust — see specs/00-overview.md. TypeScript/Node is scoped to
      # site/ only, and is installed separately there (npm), not here.
      devShells = forAllSystems (system: {
        default = (pkgsFor system).mkShell {
          packages = with (pkgsFor system); [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            nodejs_24 # for site/ only, once it exists
            jq
            git
          ];

          # Printed to stderr, not stdout: `nix develop --command cargo run
          # ... > file` must not have this banner corrupt the redirected
          # output (this bit us for real once — see git history).
          shellHook = ''
            echo "DecDev dev shell — rustc $(rustc --version), cargo $(cargo --version)" >&2
            echo "  cargo build --workspace" >&2
            echo "  cargo test --workspace" >&2
            echo "  cargo run -p decdev -- <command>" >&2
          '';
        };
      });

      formatter = forAllSystems (system: (pkgsFor system).nixfmt);
    };
}
