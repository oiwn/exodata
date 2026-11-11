# Exodata web app as a systemd service. Data and descriptions are uploaded by
# rsync (`just nix-upload-data`, `just nix-upload-descriptions`), outside Nix.
{ exodata, ... }:
{
  systemd.tmpfiles.rules = [
    "d /var/lib/exodata 0755 root root -"
    "d /var/lib/exodata/data 0755 root root -"
    "d /var/lib/exodata/content 0755 root root -"
  ];

  systemd.services.exodata = {
    description = "Exodata web app";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ];
    # Skip cleanly (instead of failing the deploy) until data is uploaded.
    unitConfig.ConditionPathExists = "/var/lib/exodata/data/stellarhosts.parquet";

    environment = {
      LEPTOS_OUTPUT_NAME = "exodata";
      LEPTOS_SITE_ROOT = "${exodata}/share/exodata/site";
      LEPTOS_SITE_PKG_DIR = "pkg";
      # Only Ferron (ferron.nix) reaches the app.
      LEPTOS_SITE_ADDR = "127.0.0.1:3000";
      LEPTOS_ENV = "PROD";
      EXO_DATA_DIR = "/var/lib/exodata/data";
      EXO_CONTENT_DIR = "/var/lib/exodata/content";
    };

    serviceConfig = {
      ExecStart = "${exodata}/bin/exodata";
      # Throwaway unprivileged user; reads the root-owned data, writes nothing.
      DynamicUser = true;
      Restart = "on-failure";
      RestartSec = 5;
    };
  };
}
