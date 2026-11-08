{ lib, ... }:
{
  imports = [ ../providers/hostinger.nix ../modules/base.nix ];

  # Preserve baseline access until a separate administrator is verified.
  services.openssh.settings.PermitRootLogin = lib.mkDefault "yes";
  # Keep the installed default hostname; the flake target is only an identifier.
  networking.hostName = "nixos";
  system.stateVersion = "26.05";
}
