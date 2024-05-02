{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
        libs = with pkgs; [
          xorg.libXi
          xorg.libX11
          xorg.libXcursor 
          libxkbcommon
          fontconfig
          # pkgs.qt5.full
          libGL

          openssl
          perl
        ];
        libPath = pkgs.lib.makeLibraryPath libs;

      in rec {
        # `nix develop`
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            rustPackages.clippy
            rust-analyzer
            slint-lsp

            pkg-config
          ] ++ libs;
          LD_LIBRARY_PATH = libPath;
        };
      });
}
