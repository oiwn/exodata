{ ... }:
{
  services.openssh.enable = true;
  # Root's key lives in /root/.ssh/authorized_keys on the server, not in Git.
  # Pin the default so a future nixpkgs cannot stop sshd reading it.
  services.openssh.authorizedKeysInHomedir = true;
}
