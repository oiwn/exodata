# Ferron in front of the app: HTTPS (Let's Encrypt, HTTP-01) for exodata.space
# and mcp.exodata.space. Pinned nixpkgs ships the package (2.7.0) but no module.
{ pkgs, ... }:
let
  config = pkgs.writeText "ferron.kdl" ''
    globals {
        // TLS-ALPN-01 cannot pass through the Cloudflare proxy.
        auto_tls_challenge "http-01"
        auto_tls_cache "/var/lib/ferron/acme"
        log_stdout #true
        error_log_stderr #true
        // No client IP and no query string (it can carry SQL); logs ship to Grafana.
        log_format "{method} {path} {status_code} {content_length} \"{header:User-Agent}\""
    }

    exodata.space {
        // MCP moved to its own subdomain; 308 keeps POST and its body.
        status 308 regex="^/mcp(?:$|[/#?])" location="https://mcp.exodata.space/"
        // Runtime downloads for `exo download`. Sibling of "/" so it does not
        // inherit the proxy (Ferron locations inherit host-level directives).
        location "/data" remove_base=#true {
            root "/var/lib/exodata/data"
            directory_listing #false
            file_cache_control "public, max-age=3600"
        }
        location "/" {
            proxy "http://127.0.0.1:3000"
            header_replace "Cache-Control" "no-cache"
            dynamic_compressed #true
        }
    }

    mcp.exodata.space {
        // Every path is the MCP endpoint; the rest of the site stays on exodata.space.
        rewrite "^/.*$" "/mcp" last=#true
        proxy "http://127.0.0.1:3000"
        // The app's MCP endpoint only accepts the exodata.space host.
        proxy_request_header_replace "Host" "exodata.space"
    }
  '';
in
{
  systemd.services.ferron = {
    description = "Ferron web server";
    wantedBy = [ "multi-user.target" ];
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];

    serviceConfig = {
      ExecStart = "${pkgs.ferron}/bin/ferron -c ${config}";
      DynamicUser = true;
      # ACME account and certificates (/var/lib/ferron/acme).
      StateDirectory = "ferron";
      AmbientCapabilities = [ "CAP_NET_BIND_SERVICE" ];
      CapabilityBoundingSet = [ "CAP_NET_BIND_SERVICE" ];
      Restart = "on-failure";
      RestartSec = 5;
    };
  };

  networking.firewall.allowedTCPPorts = [
    80
    443
  ];
}
