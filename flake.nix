{
  description = "OSRS Price CLI";

  inputs = {
   crane.url = "github:ipetkov/crane";
   crane.inputs.nixpkgs.follows = "nixpkgs";
   nixpkgs.url = "nixpkgs/nixos-unstable";
   fenix.url = "github:nix-community/fenix";
   fenix.inputs.nixpkgs.follows = "nixpkgs";
   flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system}.overrideToolchain
          fenix.packages.${system}.minimal.toolchain;

        osrs-price-cli = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;

          buildInputs = with pkgs; [
            bashInteractive
            openssl
          ]
          ++ lib.optional pkgs.stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
            libiconv
          ];

          nativeBuildInputs = with pkgs; [
            bashInteractive
            pkg-config
          ];
        };
      in {
        checks = {
          inherit osrs-price-cli;
        };

      packages.default = osrs-price-cli;

      apps.default = flake-utils.lib.mkApp {
        drv = osrs-price-cli;
      };

      devShells.default = craneLib.devShell {
        # Inherit inputs from checks.
        checks = self.checks.${system};

        # Additional dev-shell environment variables can be set directly
        # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

        # Extra inputs can be added here; cargo and rustc are provided by default.
        packages = [
          # pkgs.ripgrep
        ];
      };
    });
}
