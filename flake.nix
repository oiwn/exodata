{
  description = "Exodata NixOS infrastructure";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/08d3a97586f6b6343b3b5e799980773629432610";

  outputs = { nixpkgs, ... }: {
    nixosConfigurations.exodata-vps = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ ./infrastructure/nix/hosts/exodata-vps.nix ];
    };
  };
}
