{
  description = "Cmdflow - A CLI tool to analyze shell history with rainbow graphs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        # Сборка пакета через `nix build`
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cmdflow";
          version = "3.0.0";

          src = ./.;

          # Сюда встанет хэш твоих зависимостей (dirs, colored и их под-зависимостей).
          # При первой сборке Nix упадет и выдаст правильный sha256 — просто замени им эту заглушку.
          cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        };

        # Окружение для разработки через `nix develop`
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rust-analyzer # Для LSP в редакторе кода
          ];
        };
      });
}
