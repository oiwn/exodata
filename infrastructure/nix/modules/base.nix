{ ... }:
{
  services.openssh.enable = true;
  # Root's key lives in /root/.ssh/authorized_keys on the server, not in Git.
  # Pin the default so a future nixpkgs cannot stop sshd reading it.
  services.openssh.authorizedKeysInHomedir = true;

  nix.settings = {
    experimental-features = [
      "nix-command"
      "flakes"
    ];
    # CI builds everything and pushes to Cachix; the VPS only downloads.
    max-jobs = 0;
    # Tiny NixOS helper derivations are marked local-build-only; with no local
    # build slots they must come from the cache too.
    always-allow-substitutes = true;
    extra-substituters = [ "https://exodata.cachix.org" ];
    extra-trusted-public-keys = [
      "exodata.cachix.org-1:pgrrFg4Ejd8QJWtEpXwV2U3tXKaDtwnGFI7s05BLQiU="
    ];
  };
}
