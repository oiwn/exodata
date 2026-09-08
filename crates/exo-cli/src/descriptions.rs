pub mod prepare;
pub mod probe;

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, NaiveDateTime};
use polars::prelude::{DataFrame, ParquetReader, SerReader, StringChunked};
use serde_json::{Value, json};

pub fn columns() -> Vec<String> {
    [
        "hostname",
        "status",
        "recorded_source_date",
        "current_source_date",
        "metadata_path",
        "reason",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[derive(Default)]
struct Source {
    date: Option<NaiveDate>,
    error: Option<String>,
}

struct Metadata {
    path: PathBuf,
    date: Option<NaiveDate>,
    error: Option<String>,
}

fn date(value: &str) -> Result<Option<NaiveDate>> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    if value.len() != 10 {
        bail!("invalid date {value:?}; expected YYYY-MM-DD");
    }
    let parsed =
        NaiveDate::parse_from_str(value, "%Y-%m-%d").with_context(|| {
            format!("invalid date {value:?}; expected YYYY-MM-DD")
        })?;
    if parsed.format("%Y-%m-%d").to_string() != value {
        bail!("invalid date {value:?}; expected YYYY-MM-DD");
    }
    Ok(Some(parsed))
}

fn source_date(value: &str) -> Result<Option<NaiveDate>> {
    if value.trim().is_empty() || value.len() == 10 {
        return date(value);
    }
    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";
    let invalid = || {
        format!(
            "invalid source date {value:?}; expected YYYY-MM-DD or YYYY-MM-DD HH:MM:SS"
        )
    };
    if value.len() != 19 {
        bail!(invalid());
    }
    let parsed =
        NaiveDateTime::parse_from_str(value, FORMAT).with_context(invalid)?;
    if parsed.format(FORMAT).to_string() != value {
        bail!(invalid());
    }
    Ok(Some(parsed.date()))
}

fn report(
    hostname: Option<&str>,
    status: &str,
    recorded: Option<NaiveDate>,
    current: Option<NaiveDate>,
    path: Option<&Path>,
    reason: Option<&str>,
) -> Value {
    json!({
        "hostname": hostname,
        "status": status,
        "recorded_source_date": recorded.map(|d| d.to_string()),
        "current_source_date": current.map(|d| d.to_string()),
        "metadata_path": path.map(|p| p.to_string_lossy()),
        "reason": reason,
    })
}

/// Scan local source dates and generation artifacts without changing either.
pub fn scan(
    data_dir: &Path,
    content_dir: &Path,
    all: bool,
) -> Result<Vec<Value>> {
    let path = data_dir.join("exoplanets.parquet");
    let frame = ParquetReader::new(
        File::open(&path)
            .with_context(|| format!("cannot open {}", path.display()))?,
    )
    .with_columns(Some(vec![
        "hostname".into(),
        "rowupdate".into(),
        "releasedate".into(),
    ]))
    .finish()
    .with_context(|| {
        format!(
            "cannot read {}; required columns: hostname, rowupdate, releasedate",
            path.display()
        )
    })?;
    scan_frame(&frame, content_dir, all)
}

fn scan_frame(
    frame: &DataFrame,
    content_dir: &Path,
    all: bool,
) -> Result<Vec<Value>> {
    let strings = |name: &str| -> Result<&StringChunked> {
        frame.column(name)?.str().with_context(|| {
            format!("incompatible dataset: {name} must be a string column")
        })
    };
    let hosts = strings("hostname")?;
    let updates = strings("rowupdate")?;
    let releases = strings("releasedate")?;
    let mut sources = BTreeMap::<String, Source>::new();
    for index in 0..frame.height() {
        let Some(host) = hosts.get(index).filter(|h| !h.trim().is_empty()) else {
            bail!("invalid dataset hostname at row {}", index + 1);
        };
        let source = sources.entry(host.to_owned()).or_default();
        for (column, value) in [
            ("rowupdate", updates.get(index)),
            ("releasedate", releases.get(index)),
        ] {
            if let Some(value) = value {
                match source_date(value) {
                    Ok(value) => source.date = source.date.max(value),
                    Err(error) => {
                        source.error.get_or_insert_with(|| {
                            format!("{column} at row {}: {error}", index + 1)
                        });
                    }
                }
            }
        }
    }

    let mut metadata = BTreeMap::<String, Vec<Metadata>>::new();
    let mut rows = Vec::new();
    let entries = match fs::read_dir(content_dir) {
        Ok(entries) => Some(entries),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| {
                format!("cannot list {}", content_dir.display())
            });
        }
    };
    if let Some(entries) = entries {
        for entry in entries {
            let entry = entry.with_context(|| {
                format!("cannot list {}", content_dir.display())
            })?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path().join("metadata.toml");
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    rows.push(report(
                        None,
                        "unknown",
                        None,
                        None,
                        Some(&path),
                        Some(&format!("cannot read metadata: {error}")),
                    ));
                    continue;
                }
            };
            let value = match toml::from_str::<toml::Value>(&text) {
                Ok(value) => value,
                Err(error) => {
                    rows.push(report(
                        None,
                        "unknown",
                        None,
                        None,
                        Some(&path),
                        Some(&format!("invalid metadata: {error}")),
                    ));
                    continue;
                }
            };
            let Some(host) = value
                .get("hostname")
                .and_then(toml::Value::as_str)
                .filter(|h| !h.trim().is_empty())
            else {
                rows.push(report(
                    None,
                    "unknown",
                    None,
                    None,
                    Some(&path),
                    Some("metadata requires a nonblank string hostname"),
                ));
                continue;
            };
            if !sources.contains_key(host) {
                continue;
            }
            let parsed_date = match value.get("source_date") {
                None => Ok(None),
                Some(toml::Value::String(value)) => date(value),
                Some(_) => Err(anyhow::anyhow!(
                    "source_date must be a YYYY-MM-DD string"
                )),
            };
            let (date, error) = match parsed_date {
                Ok(date) => (date, None),
                Err(error) => (None, Some(format!("invalid metadata: {error}"))),
            };
            metadata.entry(host.to_owned()).or_default().push(Metadata {
                path,
                date,
                error,
            });
        }
    }

    for (host, source) in sources {
        let entries = metadata.remove(&host).unwrap_or_default();
        if entries.len() > 1 {
            let mut paths: Vec<_> = entries
                .iter()
                .map(|m| m.path.display().to_string())
                .collect();
            paths.sort();
            rows.push(report(
                Some(&host),
                "unknown",
                None,
                source.date,
                None,
                Some(&format!(
                    "duplicate metadata hostname: {}",
                    paths.join(", ")
                )),
            ));
            continue;
        }
        let meta = entries.first();
        let path = meta.map(|m| m.path.as_path());
        let recorded = meta.and_then(|m| m.date);
        let mut error =
            source.error.or_else(|| meta.and_then(|m| m.error.clone()));
        let mut missing = meta.is_none();
        if let Some(meta) = meta {
            let description = meta.path.parent().unwrap().join("description.md");
            match fs::read_to_string(&description) {
                Ok(text) => missing = text.trim().is_empty(),
                Err(e) if e.kind() == ErrorKind::NotFound => missing = true,
                Err(e) => {
                    error.get_or_insert_with(|| {
                        format!("cannot read description: {e}")
                    });
                }
            }
        }
        let (status, reason) = if let Some(ref error) = error {
            ("unknown", Some(error.as_str()))
        } else if missing {
            (
                "missing",
                Some(
                    "description or metadata is absent, or description is empty",
                ),
            )
        } else {
            match (source.date, recorded) {
                (Some(current), Some(recorded)) if current > recorded => (
                    "outdated",
                    Some(
                        "current source date is later than recorded source date",
                    ),
                ),
                (Some(current), Some(recorded)) if current == recorded => {
                    ("current", None)
                }
                (Some(_), Some(_)) => (
                    "unknown",
                    Some(
                        "current source date is earlier than recorded source date",
                    ),
                ),
                _ => (
                    "unknown",
                    Some("current or recorded source date is unavailable"),
                ),
            }
        };
        if all || status != "current" {
            rows.push(report(
                Some(&host),
                status,
                recorded,
                source.date,
                path,
                reason,
            ));
        }
    }
    rows.sort_by(|a, b| {
        a["hostname"]
            .as_str()
            .cmp(&b["hostname"].as_str())
            .then_with(|| {
                a["metadata_path"]
                    .as_str()
                    .cmp(&b["metadata_path"].as_str())
            })
    });
    Ok(rows)
}
