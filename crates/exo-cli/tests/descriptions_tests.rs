use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use exo_cli::descriptions::scan;
use polars::prelude::*;
use serde_json::Value;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "exodata-description-scan-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn content(&self) -> PathBuf {
        self.0.join("content")
    }

    fn data(
        &self,
        hosts: &[Option<&str>],
        updates: &[Option<&str>],
        releases: &[Option<&str>],
    ) {
        let mut frame = df!(
            "hostname" => hosts, "rowupdate" => updates, "releasedate" => releases,
            "default_flag" => vec![0i32; hosts.len()],
        ).unwrap();
        self.frame(&mut frame);
    }

    fn frame(&self, frame: &mut DataFrame) {
        ParquetWriter::new(
            fs::File::create(self.0.join("exoplanets.parquet")).unwrap(),
        )
        .finish(frame)
        .unwrap();
    }

    fn artifact(
        &self,
        directory: &str,
        metadata: Option<&str>,
        description: Option<&str>,
    ) {
        let dir = self.content().join(directory);
        fs::create_dir_all(&dir).unwrap();
        if let Some(text) = metadata {
            fs::write(dir.join("metadata.toml"), text).unwrap();
        }
        if let Some(text) = description {
            fs::write(dir.join("description.md"), text).unwrap();
        }
    }

    fn command(&self, format: &str, all: bool) -> std::process::Output {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));
        cmd.args([
            "dev",
            "descriptions",
            "scan",
            "--backend",
            "api",
            "--output",
            format,
            "--data-dir",
        ])
        .arg(&self.0)
        .arg("--content-dir")
        .arg(self.content());
        if all {
            cmd.arg("--all");
        }
        let output = cmd.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        output
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn row<'a>(rows: &'a [Value], host: &str) -> &'a Value {
    rows.iter().find(|r| r["hostname"] == host).unwrap()
}

fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>, std::time::SystemTime)> {
    let mut result = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(snapshot(&path));
        } else {
            result.push((
                path.clone(),
                fs::read(&path).unwrap(),
                fs::metadata(&path).unwrap().modified().unwrap(),
            ));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[test]
fn descriptions_statuses_grouping_and_read_only() {
    let fixture = Fixture::new();
    fixture.data(
        &[
            Some("Z"),
            Some("A"),
            Some("A"),
            Some("A"),
            Some("B"),
            Some("C"),
            Some("D"),
            Some("E"),
            Some("F"),
            Some("G"),
            Some("H"),
            Some("I"),
        ],
        &[
            None,
            Some("2026-01-01"),
            Some("2026-07-09"),
            None,
            Some("2026-08-01"),
            None,
            Some("2026-01-01"),
            None,
            Some("2026-02-30"),
            None,
            None,
            None,
        ],
        &[
            None,
            None,
            Some("2026-01-02"),
            Some("2026-07-10"),
            Some("2026-02-01"),
            Some("2026-03-01"),
            None,
            Some(" "),
            Some("2026-01-01"),
            None,
            None,
            None,
        ],
    );
    fixture.artifact("unrelated-id", Some("hostname = 'A'\nsource_date = '2026-07-10'\ngenerated_at = 'irrelevant'\nmodel = 'anything'"), Some("Article"));
    fixture.artifact(
        "b",
        Some("hostname = 'B'\nsource_date = '2026-07-10'"),
        Some("Article"),
    );
    fixture.artifact(
        "c",
        Some("hostname = 'C'\nsource_date = '2026-04-01'"),
        Some("Article"),
    );
    fixture.artifact("d", Some("hostname = 'D'"), Some("Article"));
    fixture.artifact(
        "e",
        Some("hostname = 'E'\nsource_date = '2026-01-01'"),
        Some("Article"),
    );
    fixture.artifact("g", Some("hostname = 'G'"), Some(" \n"));
    fixture.artifact("h", Some("hostname = 'H'"), None);
    fixture.artifact("I", None, Some("Article without metadata"));
    fs::write(
        fixture.content().join("unrelated-id/request.toml"),
        "not parsed [",
    )
    .unwrap();
    let before = snapshot(&fixture.0);
    let rows = scan(&fixture.0, &fixture.content(), true).unwrap();
    assert_eq!(rows.len(), 10);
    for (host, status) in [
        ("A", "current"),
        ("B", "outdated"),
        ("C", "unknown"),
        ("D", "unknown"),
        ("E", "unknown"),
        ("F", "unknown"),
        ("G", "missing"),
        ("H", "missing"),
        ("I", "missing"),
        ("Z", "missing"),
    ] {
        assert_eq!(row(&rows, host)["status"], status, "{host}");
    }
    assert_eq!(row(&rows, "A")["current_source_date"], "2026-07-10");
    assert_eq!(row(&rows, "B")["current_source_date"], "2026-08-01");
    assert_eq!(row(&rows, "F")["current_source_date"], "2026-01-01");
    assert!(row(&rows, "Z")["metadata_path"].is_null());
    assert!(row(&rows, "Z")["current_source_date"].is_null());
    assert!(
        rows.windows(2)
            .all(|pair| pair[0]["hostname"].as_str()
                < pair[1]["hostname"].as_str())
    );
    let filtered = scan(&fixture.0, &fixture.content(), false).unwrap();
    assert_eq!(filtered.len(), rows.len() - 1);
    assert!(filtered.iter().all(|r| r["status"] != "current"));
    assert_eq!(before, snapshot(&fixture.0));
}

#[test]
fn descriptions_invalid_metadata_and_ambiguity() {
    let fixture = Fixture::new();
    fixture.data(
        &[Some("A"), Some("B"), Some("C"), Some("D")],
        &[Some("2026-01-01"); 4],
        &[None; 4],
    );
    fixture.artifact(
        "one",
        Some("hostname = 'A'\nsource_date = '2026-01-01'"),
        Some("Article"),
    );
    fixture.artifact("two", Some("hostname = 'A'"), None);
    fixture.artifact(
        "bad-date",
        Some("hostname = 'B'\nsource_date = '2026-02-30'"),
        None,
    );
    fixture.artifact(
        "bad-type",
        Some("hostname = 'C'\nsource_date = 42"),
        Some("Article"),
    );
    fixture.artifact("D", Some("hostname = 'D'\nbroken = ["), Some("Article"));
    fixture.artifact("no-host", Some("source_date = '2026-01-01'"), None);
    fixture.artifact("removed", Some("hostname = 'Removed'"), None);
    let rows = scan(&fixture.0, &fixture.content(), true).unwrap();
    assert_eq!(rows.len(), 6);
    assert!(rows[0]["hostname"].is_null());
    assert!(rows[1]["hostname"].is_null());
    assert!(
        rows[0]["metadata_path"].as_str() < rows[1]["metadata_path"].as_str()
    );
    for host in ["A", "B", "C"] {
        assert_eq!(row(&rows, host)["status"], "unknown");
    }
    assert!(
        row(&rows, "A")["reason"]
            .as_str()
            .unwrap()
            .contains("duplicate")
    );
    assert!(row(&rows, "A")["metadata_path"].is_null());
    assert_eq!(row(&rows, "D")["status"], "missing");
}

#[test]
fn descriptions_formats_filtering_and_defaults() {
    let fixture = Fixture::new();
    fixture.data(
        &[Some("B"), Some("A")],
        &[Some("2026-01-01"); 2],
        &[None; 2],
    );
    fixture.artifact(
        "one",
        Some("hostname = 'A'\nsource_date = '2026-01-01'"),
        Some("Article"),
    );
    let before = snapshot(&fixture.0);
    for all in [false, true] {
        let expected = scan(&fixture.0, &fixture.content(), all).unwrap();
        let json: Vec<Value> =
            serde_json::from_slice(&fixture.command("json", all).stdout).unwrap();
        assert_eq!(json, expected);
        let csv = fixture.command("csv", all).stdout;
        let mut reader = csv::Reader::from_reader(csv.as_slice());
        let headers = reader.headers().unwrap().clone();
        assert_eq!(
            headers.iter().collect::<Vec<_>>(),
            exo_cli::descriptions::columns()
        );
        let records: Vec<_> = reader.records().map(Result::unwrap).collect();
        assert_eq!(records.len(), expected.len());
        let table =
            String::from_utf8(fixture.command("table", all).stdout).unwrap();
        for (record, row) in records.iter().zip(&expected) {
            for (column, value) in headers.iter().zip(record.iter()) {
                assert_eq!(value, row[column].as_str().unwrap_or(""));
                if !value.is_empty() {
                    assert!(table.contains(value), "{value}");
                }
            }
        }
        assert_eq!(
            table.lines().filter(|line| line.starts_with('|')).count(),
            expected.len() + 2
        );
    }
    assert_eq!(snapshot(&fixture.0), before);

    // Defaults resolve from the working directory without creating content.
    fs::create_dir(fixture.0.join("data")).unwrap();
    fs::rename(
        fixture.0.join("exoplanets.parquet"),
        fixture.0.join("data/exoplanets.parquet"),
    )
    .unwrap();
    let output = Command::new(assert_cmd::cargo::cargo_bin!("exodata"))
        .current_dir(&fixture.0)
        .args(["dev", "descriptions", "scan", "--output", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let rows: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert!(rows.iter().all(|row| row["status"] == "missing"));
    assert!(!fixture.0.join("content/systems").exists());
}

#[test]
fn descriptions_fatal_input_errors() {
    let fixture = Fixture::new();
    assert!(scan(&fixture.0, &fixture.content(), false).is_err());
    fs::write(fixture.0.join("exoplanets.parquet"), "invalid").unwrap();
    assert!(scan(&fixture.0, &fixture.content(), false).is_err());
    fixture.frame(&mut df!("hostname" => ["A"]).unwrap());
    let error = scan(&fixture.0, &fixture.content(), false).unwrap_err();
    assert!(error.to_string().contains("required columns"));
    fixture.frame(
        &mut df!("hostname" => ["A"], "rowupdate" => [1], "releasedate" => [""])
            .unwrap(),
    );
    assert!(
        scan(&fixture.0, &fixture.content(), false)
            .unwrap_err()
            .to_string()
            .contains("rowupdate must be a string")
    );
    for host in [None, Some(""), Some(" \n")] {
        fixture.data(&[host], &[None], &[None]);
        assert!(
            scan(&fixture.0, &fixture.content(), false)
                .unwrap_err()
                .to_string()
                .contains("invalid dataset hostname")
        );
    }
    fixture.data(&[Some("A")], &[None], &[None]);
    assert_eq!(
        scan(&fixture.0, &fixture.content(), false).unwrap()[0]["status"],
        "missing"
    );
    assert!(!fixture.content().exists());
    fs::write(fixture.content(), "not a directory").unwrap();
    assert!(
        scan(&fixture.0, &fixture.content(), false)
            .unwrap_err()
            .to_string()
            .contains("cannot list")
    );
}

#[test]
fn descriptions_nasa_timestamps_use_calendar_dates() {
    let fixture = Fixture::new();
    fixture.data(
        &[Some("K2-316"), Some("K2-316"), Some("B")],
        &[Some("2020-09-09"), None, Some("2024-02-29 23:59:59")],
        &[
            Some("2020-09-09 17:05:56"),
            Some("2020-09-10 00:00:01"),
            None,
        ],
    );
    let rows = scan(&fixture.0, &fixture.content(), true).unwrap();
    assert_eq!(row(&rows, "K2-316")["status"], "missing");
    assert_eq!(row(&rows, "K2-316")["current_source_date"], "2020-09-10");
    assert_eq!(row(&rows, "B")["current_source_date"], "2024-02-29");
    fixture.artifact(
        "one",
        Some("hostname = 'K2-316'\nsource_date = '2020-09-10'"),
        Some("Article"),
    );
    assert_eq!(
        row(
            &scan(&fixture.0, &fixture.content(), true).unwrap(),
            "K2-316"
        )["status"],
        "current"
    );
    fixture.artifact(
        "one",
        Some("hostname = 'K2-316'\nsource_date = '2020-09-10 00:00:01'"),
        Some("Article"),
    );
    assert_eq!(
        row(
            &scan(&fixture.0, &fixture.content(), true).unwrap(),
            "K2-316"
        )["status"],
        "unknown"
    );
}

#[test]
fn descriptions_calendar_validation() {
    let fixture = Fixture::new();
    for invalid in [
        "2026-02-29",
        "2026-2-01",
        "2026-01-1",
        " 2026-01-01",
        "2026-01-01T00:00:00Z",
        "2026-13-01",
        "+10000-01-01",
        "2026-02-30 17:05:56",
        "2026-01-01 24:00:00",
        "2026-01-01 17:60:00",
        "2026-01-01 17:05:99",
        "2026-01-01 7:05:56",
        "2026-01-01 17:05:56 garbage",
    ] {
        fixture.data(&[Some("A")], &[Some(invalid)], &[None]);
        let rows = scan(&fixture.0, &fixture.content(), false).unwrap();
        assert_eq!(rows[0]["status"], "unknown", "{invalid}");
        assert!(rows[0]["current_source_date"].is_null());
    }
    fixture.data(&[Some("A")], &[Some("2024-02-29")], &[None]);
    assert_eq!(
        scan(&fixture.0, &fixture.content(), false).unwrap()[0]["current_source_date"],
        "2024-02-29"
    );
}
