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

          shellHook = ''
            echo "DecDev dev shell — rustc $(rustc --version), cargo $(cargo --version)"
            echo "  cargo build --workspace"
            echo "  cargo test --workspace"
            echo "  cargo run -p cli -- <command>   # once cli/ exists"
          '';
        };
      });

      formatter = forAllSystems (system: (pkgsFor system).nixfmt);
    };
}
