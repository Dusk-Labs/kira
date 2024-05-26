{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    nix-github-actions.url = "github:IGI-111/nix-github-actions?ref=IGI-111/add-aarch64-darwin";
    nix-github-actions.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      nix-github-actions,
      ...
    }:
    let
      buildPackageForSystem =
        {
          pkgs,
          buildInputs,
          nativeBuildInputs,
        }:
        let
          craneLib = crane.mkLib pkgs;
          libPath = pkgs.lib.makeLibraryPath buildInputs;
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
          cargoArtifacts = craneLib.buildDepsOnly {
            inherit src buildInputs nativeBuildInputs;
            LD_LIBRARY_PATH = libPath;
          };
          kira = craneLib.buildPackage {
            inherit
              src
              buildInputs
              nativeBuildInputs
              cargoArtifacts
              ;
            doNotSign = true; # FIXME: disable apple binary signature
            LD_LIBRARY_PATH = libPath;
          };
        in
        {
          defaultPackage = kira;
          checks = {
            inherit kira;
            kira-clippy = craneLib.cargoClippy {
              inherit
                src
                buildInputs
                nativeBuildInputs
                cargoArtifacts
                ;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            };
            kira-fmt = craneLib.cargoFmt { inherit src; };
          };
          devShell = pkgs.mkShell {
            inherit buildInputs nativeBuildInputs;
            LD_LIBRARY_PATH = libPath;
          };
        };

      buildPackageDeps = {
        "x86_64-linux" =
          let
            pkgs = nixpkgs.legacyPackages.x86_64-linux;
          in
          {
            inherit pkgs;
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
          };

        "aarch64-darwin" =
          let
            pkgs = nixpkgs.legacyPackages.aarch64-darwin;
          in
          {
            inherit pkgs;
            buildInputs = with pkgs; [
              openssl
              darwin.signingUtils
              darwin.libiconv
              darwin.libobjc
              # darwin.xcode
              # darwin.apple_sdk.sdk
              darwin.apple_sdk.Libsystem
              # darwin.apple_sdk.frameworks
              darwin.apple_sdk.frameworks.SystemConfiguration
              darwin.apple_sdk.frameworks.AppKit
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.Cocoa
              darwin.apple_sdk.frameworks.Security
            ];
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              rustfmt
              rustPackages.clippy
              rust-analyzer
              slint-lsp

              pkg-config
              uutils-coreutils
              findutils
            ];
          };
      };
    in
    {
      githubActions = nix-github-actions.lib.mkGithubMatrix { inherit (self) checks; };
    }
    // flake-utils.lib.eachSystem [
      "x86_64-linux"
      # "x86_64-darwin"
      "aarch64-darwin"
    ] (system: buildPackageForSystem buildPackageDeps."${system}");
}
