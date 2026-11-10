# Captured from root@nixos-vps:/etc/nixos/configuration.nix, 2026-11-08.
# Installed nixpkgs: 08d3a97586f6b6343b3b5e799980773629432610.
# Hostinger KVM template: preserve provider boot and provisioning behavior.
{ lib, pkgs, modulesPath, ... }:
{
  imports = [ "${modulesPath}/profiles/qemu-guest.nix" ];

  boot.loader.grub.enable = true;
  boot.loader.grub.device = "/dev/sda";
  boot.loader.grub.efiSupport = false;
  boot.loader.timeout = 1;
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
    autoResize = true;
  };
  boot.growPartition = true;
  boot.kernelParams = [
    "console=tty0" "console=ttyS0,115200" "earlyprintk=ttyS0,115200"
    "consoleblank=0" "memhp_default_state=online"
  ];

  services.qemuGuest.enable = true;
  services.cloud-init.enable = true;
  services.cloud-init.network.enable = true;
  services.cloud-init.settings.datasource_list = [ "NoCloud" "ConfigDrive" ];
  # The template does not ship write-ssh-key-fingerprints.
  services.cloud-init.settings.ssh.emit_keys_to_console = false;
  # Cloud-init owns networking; DHCP would race it.
  networking.useDHCP = lib.mkForce false;

  # Hostinger invokes these tools through the guest agent during provisioning.
  environment.systemPackages = with pkgs; [
    cloud-init dnsutils nettools curl wget nano
  ];
  systemd.services.qemu-guest-agent.environment.PATH =
    lib.mkForce "/run/wrappers/bin:/run/current-system/sw/bin:/bin";
}
