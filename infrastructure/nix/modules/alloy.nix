# Grafana Alloy: host metrics -> Grafana Cloud Prometheus, journal logs of our
# units -> Grafana Cloud Loki. Credentials come from a root-only env file pushed
# by `just nix-push-secrets`; nothing secret is in this config or the store.
{ ... }:
{
  services.alloy = {
    enable = true;
    environmentFile = "/var/lib/secrets/alloy.env";
    extraFlags = [ "--disable-reporting" ];
  };

  environment.etc."alloy/config.alloy".text = ''
    // ---- Metrics ----
    prometheus.exporter.unix "host" { }

    // Labels expected by the Grafana Cloud "Linux Server" integration dashboards.
    discovery.relabel "host" {
      targets = prometheus.exporter.unix.host.targets

      rule {
        target_label = "job"
        replacement  = "integrations/node_exporter"
      }
      rule {
        target_label = "instance"
        replacement  = "exodata-vps"
      }
    }

    prometheus.scrape "host" {
      targets         = discovery.relabel.host.output
      scrape_interval = "60s"
      forward_to      = [prometheus.remote_write.grafana.receiver]
    }

    prometheus.remote_write "grafana" {
      endpoint {
        url = sys.env("GRAFANA_PROM_URL")

        basic_auth {
          username = sys.env("GRAFANA_PROM_USER")
          password = sys.env("GRAFANA_TOKEN")
        }
      }
    }

    // ---- Logs ----
    loki.relabel "journal" {
      forward_to = []

      rule {
        source_labels = ["__journal__systemd_unit"]
        target_label  = "unit"
      }
      rule {
        source_labels = ["__journal_priority_keyword"]
        target_label  = "level"
      }
    }

    loki.source.journal "units" {
      matches       = "_SYSTEMD_UNIT=exodata.service _SYSTEMD_UNIT=ferron.service _SYSTEMD_UNIT=alloy.service"
      // Skip older journal entries on first start (they predate the IP-free access log).
      max_age       = "1m"
      relabel_rules = loki.relabel.journal.rules
      labels        = { host = "exodata-vps" }
      forward_to    = [loki.write.grafana.receiver]
    }

    loki.write "grafana" {
      endpoint {
        url = sys.env("GRAFANA_LOKI_URL")

        basic_auth {
          username = sys.env("GRAFANA_LOKI_USER")
          password = sys.env("GRAFANA_TOKEN")
        }
      }
    }
  '';
}
