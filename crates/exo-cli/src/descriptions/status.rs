//! Local generation status: per-system state, usage totals, and failure
//! tracking without any model calls.
use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Value, json};

pub fn columns() -> Vec<String> {
    [
        "hostname",
        "state",
        "fingerprint_version",
        "attempts",
        "prompt_tokens",
        "completion_tokens",
        "total_tokens",
        "critic_findings",
        "generated_at",
        "error",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn read_toml(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    Ok(serde_json::to_value(
        toml::from_str::<toml::Value>(&text)
            .with_context(|| format!("Invalid TOML in {}", path.display()))?,
    )?)
}

fn text(value: &Value, key: &str) -> Value {
    value[key].as_str().map_or(Value::Null, |s| json!(s))
}

fn number(value: &Value, key: &str) -> Value {
    value[key].clone()
}

/// Report per-system generation state across a content directory.
/// State precedence: `failed` (a `fail.toml` records the latest failed
/// attempt, even when an older description survives), `generated`
/// (nonempty `description.md`), then `missing`. A summary goes to stderr.
pub fn run(content_dir: &Path) -> Result<Vec<Value>> {
    let mut systems = BTreeMap::new();
    for entry in fs::read_dir(content_dir)
        .with_context(|| format!("Cannot read {}", content_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let directory = entry.path();
        if !directory.join("request.toml").try_exists()? {
            continue;
        }
        let request = read_toml(&directory.join("request.toml"))?;
        let hostname = request["system"]["hostname"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                directory
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
        let metadata = if directory.join("metadata.toml").try_exists()? {
            Some(read_toml(&directory.join("metadata.toml"))?)
        } else {
            None
        };
        let failure = if directory.join("fail.toml").try_exists()? {
            Some(read_toml(&directory.join("fail.toml"))?)
        } else {
            None
        };
        let description = directory.join("description.md");
        let state = if failure.is_some() {
            "failed"
        } else if description.try_exists()?
            && !fs::read_to_string(&description)?.trim().is_empty()
        {
            "generated"
        } else {
            "missing"
        };
        let m = metadata.as_ref();
        let f = failure.as_ref();
        systems.insert(
            hostname.clone(),
            json!({
                "hostname": hostname,
                "state": state,
                "fingerprint_version": m
                    .map(|m| number(m, "fingerprint_version"))
                    .unwrap_or(Value::Null),
                "attempts": m
                    .map(|m| number(m, "attempts"))
                    .unwrap_or(Value::Null),
                "prompt_tokens": m
                    .map(|m| number(m, "prompt_tokens"))
                    .unwrap_or(Value::Null),
                "completion_tokens": m
                    .map(|m| number(m, "completion_tokens"))
                    .unwrap_or(Value::Null),
                "total_tokens": m
                    .map(|m| number(m, "total_tokens"))
                    .unwrap_or(Value::Null),
                "critic_findings": m
                    .and_then(|m| m["critic"]["findings"].as_array())
                    .map(|list| json!(list.len()))
                    .unwrap_or(Value::Null),
                "generated_at": m
                    .map(|m| text(m, "generated_at"))
                    .unwrap_or(Value::Null),
                "error": f
                    .map(|f| text(f, "error"))
                    .unwrap_or(Value::Null),
            }),
        );
    }
    let rows: Vec<Value> = systems.into_values().collect();
    let count = |state: &str| rows.iter().filter(|r| r["state"] == state).count();
    let tokens: u64 =
        rows.iter().filter_map(|r| r["total_tokens"].as_u64()).sum();
    eprintln!(
        "{} systems; {} generated, {} failed, {} missing; recorded tokens: {tokens}",
        rows.len(),
        count("generated"),
        count("failed"),
        count("missing"),
    );
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(std::path::PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "exodata-status-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn system(
        root: &TestDirectory,
        id: &str,
        description: Option<&str>,
        failure: Option<&str>,
        metadata: Option<&str>,
    ) {
        let dir = root.0.join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("request.toml"),
            format!("[system]\nhostname = '{id}'\n"),
        )
        .unwrap();
        if let Some(text) = description {
            fs::write(dir.join("description.md"), text).unwrap();
        }
        if let Some(text) = metadata {
            fs::write(dir.join("metadata.toml"), text).unwrap();
        }
        if let Some(error) = failure {
            fs::write(dir.join("fail.toml"), format!("error = '{error}'\n"))
                .unwrap();
        }
    }

    #[test]
    fn states_follow_failure_precedence_and_report_usage() {
        let root = TestDirectory::new();
        system(
            &root,
            "alpha",
            Some("# Alpha\n\nText."),
            None,
            Some(
                "hostname = 'Alpha'\nfingerprint_version = 4\nattempts = 2\n\
                 prompt_tokens = 10\ncompletion_tokens = 5\ntotal_tokens = 15\n\
                 generated_at = '2026-09-10T08:00:00+00:00'\n\
                 [critic]\nfindings = []\n",
            ),
        );
        system(
            &root,
            "beta",
            Some("# Beta\n\nOld text."),
            Some("draft validation failed after 3 attempts"),
            Some("hostname = 'Beta'\nfingerprint_version = 4\nattempts = 3\n"),
        );
        system(&root, "gamma", None, None, None);
        let rows = run(&root.0).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["hostname"], "alpha");
        assert_eq!(rows[0]["state"], "generated");
        assert_eq!(rows[0]["fingerprint_version"], 4);
        assert_eq!(rows[0]["total_tokens"], 15);
        assert_eq!(rows[0]["critic_findings"], 0);
        assert_eq!(rows[1]["state"], "failed");
        assert!(rows[1]["error"].as_str().unwrap().contains("3 attempts"));
        assert_eq!(rows[1]["generated_at"], Value::Null);
        assert_eq!(rows[2]["state"], "missing");
        assert_eq!(rows[2]["fingerprint_version"], Value::Null);
    }
}
