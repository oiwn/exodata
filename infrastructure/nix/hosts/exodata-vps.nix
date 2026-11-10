{ lib, ... }:
{
  imports = [
    ../providers/hostinger.nix
    ../modules/base.nix
  ];

  # Keep SSH login exactly as installed.
  services.openssh.settings.PermitRootLogin = lib.mkDefault "yes";
  networking.hostName = "exodata-vps";
  system.stateVersion = "26.05";
}
