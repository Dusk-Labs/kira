{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        buildInputs = with pkgs; [
          xorg.libXi
          xorg.libX11
          xorg.libXcursor
          libxkbcommon
          fontconfig
          libGL
          wayland
          openssl
        ];
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          rustfmt
          rustPackages.clippy
          rust-analyzer
          slint-lsp

          pkg-config
          # kdialog
        ];
        libPath = pkgs.lib.makeLibraryPath buildInputs;
      in
      {
        packages.default = craneLib.buildPackage {
          src =
            let
              # Only keeps slint files
              dirFilter = path: _type: builtins.match ".*(assets|src|ui)/.*$" path != null;
              slintFilter = path: _type: builtins.match ".*slint$" path != null;
              sourceFilter =
                path: type:
                (dirFilter path type) || (slintFilter path type) || (craneLib.filterCargoSources path type);
            in
            nixpkgs.lib.cleanSourceWith {
              src = craneLib.path ./.;
              filter = sourceFilter;
            };

          # Add extra inputs here or any other derivation settings
          # doCheck = true;
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
          LD_LIBRARY_PATH = libPath;
        };
        # `nix develop`
        devShell = pkgs.mkShell {
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
          LD_LIBRARY_PATH = libPath;
        };
      }
    );
}
