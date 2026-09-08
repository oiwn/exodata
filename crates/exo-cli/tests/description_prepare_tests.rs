use polars::prelude::*;
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "exodata-prepare-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir_all(&directory).unwrap();
        let fixture = Self(directory);
        fixture.data(&["Test Host"], &[1]);
        fixture
    }

    fn data(&self, names: &[&str], defaults: &[i32]) {
        let mut hosts = df!("hostname" => names, "st_mass" => vec![1.0; names.len()], "st_masslim" => vec![0i32; names.len()]).unwrap();
        let mut planets = df!("hostname" => names, "pl_name" => vec!["Test b"; names.len()],
            "default_flag" => defaults, "pl_orbper" => vec![10.0; names.len()], "pl_orbperlim" => vec![0i32; names.len()]).unwrap();
        for (name, frame) in
            [("stellarhosts", &mut hosts), ("exoplanets", &mut planets)]
        {
            ParquetWriter::new(
                fs::File::create(self.0.join(format!("{name}.parquet"))).unwrap(),
            )
            .finish(frame)
            .unwrap();
        }
    }

    fn command(&self, args: &[&str]) -> Output {
        Command::new(assert_cmd::cargo::cargo_bin!("exodata"))
            .current_dir(&self.0)
            .args([
                "dev",
                "descriptions",
                "prepare",
                "--hostname",
                "Test Host",
                "--data-dir",
            ])
            .arg(&self.0)
            .args(args)
            .output()
            .unwrap()
    }

    fn directory(&self) -> PathBuf {
        self.0.join("content/systems/test-host")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn default_paths_parseable_artifacts_and_safe_force() {
    let f = Fixture::new();
    let first = f.command(&["--output", "json"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(
        report[0]["request_path"],
        "content/systems/test-host/request.toml"
    );
    let request = fs::read_to_string(f.directory().join("request.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&request).unwrap();
    assert_eq!(parsed["system"]["hostname"].as_str(), Some("Test Host"));
    let evidence = fs::read(f.directory().join("evidence.json")).unwrap();
    let parsed: Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["hostname"], "Test Host");
    fs::write(f.directory().join("description.md"), "Manual article").unwrap();
    fs::write(
        f.directory().join("metadata.toml"),
        "hostname = 'Test Host'\nsource_date = '2024-01-01'\n",
    )
    .unwrap();
    let refused = f.command(&[]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--force"));
    assert_eq!(
        fs::read(f.directory().join("evidence.json")).unwrap(),
        evidence
    );
    let forced = f.command(&["--force", "--output", "csv"]);
    assert!(
        forced.status.success(),
        "{}",
        String::from_utf8_lossy(&forced.stderr)
    );
    assert!(String::from_utf8_lossy(&forced.stdout).starts_with("hostname,"));
    assert_eq!(
        fs::read_to_string(f.directory().join("request.toml")).unwrap(),
        request
    );
    assert_eq!(
        fs::read_to_string(f.directory().join("description.md")).unwrap(),
        "Manual article"
    );
    assert_eq!(
        fs::read_to_string(f.directory().join("metadata.toml")).unwrap(),
        "hostname = 'Test Host'\nsource_date = '2024-01-01'\n"
    );
    assert!(!f.directory().join("system.txt").exists());
}

#[test]
fn custom_output_directory_and_stored_identity_mismatch() {
    let f = Fixture::new();
    assert!(f.command(&["--output-dir", "custom"]).status.success());
    assert!(!f.directory().exists());
    let dir = f.0.join("custom/test-host");
    fs::write(dir.join("metadata.toml"), "hostname = 'Different'\n").unwrap();
    let original = fs::read(dir.join("request.toml")).unwrap();
    let output = f.command(&["--output-dir", "custom", "--force"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("hostname mismatch")
    );
    assert_eq!(fs::read(dir.join("request.toml")).unwrap(), original);
}

#[test]
fn collisions_and_failed_selection_write_nothing() {
    let f = Fixture::new();
    f.data(&["Test Host", "Test-Host"], &[1, 1]);
    let output = f.command(&[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("collides"));
    assert!(!f.directory().exists());
    f.data(&["Test Host"], &[0]);
    let output = f.command(&[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unique default"));
    assert!(!f.directory().exists());
}
