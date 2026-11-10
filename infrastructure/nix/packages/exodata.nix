# Exodata web app: Leptos SSR server plus the split WASM site, built with crane.
# Built only in GitHub CI and pushed to the exodata Cachix cache.
{
  lib,
  pkgs,
  crane,
  buildDate,
  buildWasmBindgenCli,
  fetchCrate,
  rustPlatform,
  cargo-leptos,
  binaryen,
  tailwindcss_4,
  pkg-config,
  cmake,
  perl,
}:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain (
    p: p.rust-bin.stable."1.99.0".default.override { targets = [ "wasm32-unknown-unknown" ]; }
  );

  # Must match the wasm-bindgen version in Cargo.lock; pinned nixpkgs stops at 0.2.126.
  wasm-bindgen-cli = buildWasmBindgenCli rec {
    src = fetchCrate {
      pname = "wasm-bindgen-cli";
      version = "0.2.127";
      hash = "sha256-di+qBAdd7pENLiIB9CoZoab+W5xeDoByMREcCGTSzWo=";
    };
    cargoDeps = rustPlatform.fetchCargoVendor {
      inherit src;
      inherit (src) pname version;
      hash = "sha256-FTv2GZIAQs0ePdIZXIXil7JbZ6kIT05VG6vqC1qNFxQ=";
    };
  };

  root = ../../..;
  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions (
      map (p: root + p) [
        "/Cargo.toml"
        "/Cargo.lock"
        "/.cargo"
        "/src"
        "/crates"
        "/docs"
        "/style"
        "/public"
        "/locales"
        "/content"
      ]
    );
  };

  common = {
    inherit src;
    pname = "exodata";
    inherit (craneLib.crateNameFromCargoToml { cargoToml = root + "/Cargo.toml"; }) version;
    strictDeps = true;
    doCheck = false;
    nativeBuildInputs = [
      pkg-config
      cmake
      perl
    ];
  };

  # Mirror cargo-leptos's server and WASM cargo invocations (same flags, target
  # dirs, and RUSTFLAGS) so source-only changes reuse these dependency builds.
  cargoArtifacts = craneLib.buildDepsOnly (
    common
    // {
      buildPhaseCargoCommand = ''
        cargo build --locked --release --package=exodata-web --bin=exodata-web \
          --no-default-features --features=ssr
        RUSTFLAGS="-Clink-args=--emit-relocs" cargo build --locked --package=exodata-web --lib \
          --target-dir=target/front --target=wasm32-unknown-unknown \
          --no-default-features --features=hydrate --profile=wasm-release
      '';
    }
  );
in
craneLib.buildPackage (
  common
  // {
    inherit cargoArtifacts;
    # cargo-leptos uses these from PATH instead of downloading them.
    nativeBuildInputs = common.nativeBuildInputs ++ [
      cargo-leptos
      wasm-bindgen-cli
      binaryen
      tailwindcss_4
    ];

    # Commit time of the flake source (YYYYMMDDHHMMSS), shown in the footer.
    BUILD_DATE = "${lib.substring 0 4 buildDate}-${lib.substring 4 2 buildDate}-${lib.substring 6 2 buildDate}";
    BUILD_TIMESTAMP = "${lib.substring 0 4 buildDate}-${lib.substring 4 2 buildDate}-${lib.substring 6 2 buildDate} ${lib.substring 8 2 buildDate}:${lib.substring 10 2 buildDate} UTC";

    buildPhaseCargoCommand = "cargo leptos build --release --split";
    # cargo-leptos builds, so crane has no cargo log to install from; we install below.
    doNotPostBuildInstallCargoBinaries = true;
    installPhaseCommand = ''
      install -Dm755 target/release/exodata-web $out/bin/exodata
      mkdir -p $out/share/exodata
      cp -r target/site $out/share/exodata/site
    '';
  }
)
