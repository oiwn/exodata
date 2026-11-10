{
  description = "Exodata NixOS infrastructure";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/08d3a97586f6b6343b3b5e799980773629432610";
  # crane v0.24.0
  inputs.crane.url = "github:ipetkov/crane/32daa78aa882b7a43c508d1d5a5ad18a7968d731";
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay/b70538330f20adc9665afa2976471eb5b85ebd1f";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, crane, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      exodata = pkgs.callPackage ./infrastructure/nix/packages/exodata.nix {
        inherit crane;
        buildDate = self.lastModifiedDate;
      };
    in {
      packages.${system} = {
        inherit exodata;
        default = exodata;
      };

      nixosConfigurations.exodata-vps = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [ ./infrastructure/nix/hosts/exodata-vps.nix ];
      };
    };
}
