{
  description = "(Neo)vim perceptual color scheme compiler";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      rec {
        packages.termsnap = pkgs.callPackage ./default.nix { };
        packages.default = packages.termsnap;
        devShells.default = pkgs.mkShell
          {
            buildInputs = with pkgs; [
              cargo
              clippy
              rust-analyzer
              rustc
              rustfmt
              bashInteractive

              # Programs for Termsnap examples
              cowsay
              tokei
            ];
          };
      }
    );
}
