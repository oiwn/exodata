//! Retroactive application of the algorithmic article normalizers
//! (measurement bolding and the density possessive) to stored
//! descriptions, without model calls.
use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use super::validate;

pub fn columns() -> Vec<String> {
    ["hostname", "path", "changed"]
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

/// Apply `validate::normalize_article` to every stored description,
/// rewriting files that change. The spectral label comes from the saved
/// request when present.
pub fn run(content_dir: &Path) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(content_dir)
        .with_context(|| format!("Cannot read {}", content_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let directory = entry.path();
        let description_path = directory.join("description.md");
        if !description_path.try_exists()? {
            continue;
        }
        let hostname = read_toml(&directory.join("request.toml"))
            .ok()
            .and_then(|request| {
                request["system"]["hostname"].as_str().map(str::to_owned)
            })
            .or_else(|| {
                directory
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .unwrap_or_default();
        let spectral_label = read_toml(&directory.join("request.toml"))
            .ok()
            .and_then(|request| {
                request["star"]["spectral_type"].as_str().map(str::to_owned)
            });
        let text = fs::read_to_string(&description_path)?;
        let normalized =
            validate::normalize_article(&text, spectral_label.as_deref());
        let changed = normalized != text;
        if changed {
            fs::write(&description_path, &normalized).with_context(|| {
                format!("Cannot rewrite {}", description_path.display())
            })?;
        }
        rows.push(json!({"hostname": hostname,
            "path": description_path, "changed": changed}));
    }
    rows.sort_by(|a, b| a["hostname"].as_str().cmp(&b["hostname"].as_str()));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(std::path::PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "exodata-normalize-{}-{}",
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

    #[test]
    fn rewrites_possessives_and_bolds_but_skips_clean_files() {
        let root = TestDirectory::new();
        let system = root.0.join("alpha");
        fs::create_dir(&system).unwrap();
        fs::write(
            system.join("request.toml"),
            "[system]\nhostname = 'Alpha'\n[star]\nspectral_type = 'G2 V'\n",
        )
        .unwrap();
        fs::write(
            system.join("description.md"),
            "# Alpha\n\nThe value is 0.51 times Earth.",
        )
        .unwrap();
        let rows = run(&root.0).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["changed"], true);
        let rewritten =
            fs::read_to_string(system.join("description.md")).unwrap();
        assert!(rewritten.contains("**0.51 times Earth's**"));
        let rows = run(&root.0).unwrap();
        assert_eq!(rows[0]["changed"], false, "second pass is a no-op");
        let other = root.0.join("no-description");
        fs::create_dir(&other).unwrap();
        let rows = run(&root.0).unwrap();
        assert_eq!(rows.len(), 1, "directories without articles are skipped");
    }
}
