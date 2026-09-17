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
/// rewriting files that change. The spectral label and planet names
/// come from the saved request when present.
pub fn run(content_dir: &Path, label: Option<&str>) -> Result<Vec<Value>> {
    if let Some(label) = label {
        super::batch::validate_label(label)?;
    }
    let filename = label.map_or_else(
        || "description.md".to_owned(),
        |label| format!("description_{label}.md"),
    );
    let mut rows = Vec::new();
    for entry in fs::read_dir(content_dir)
        .with_context(|| format!("Cannot read {}", content_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let directory = entry.path();
        let description_path = directory.join(&filename);
        if !description_path.try_exists()? {
            continue;
        }
        let request = read_toml(&directory.join("request.toml"))
            .ok()
            .unwrap_or(Value::Null);
        let hostname = request["system"]["hostname"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| {
                directory
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .unwrap_or_default();
        let spectral_label =
            request["star"]["spectral_type"].as_str().map(str::to_owned);
        let planet_names: Vec<String> = request["planets"]
            .as_array()
            .map(|planets| {
                planets
                    .iter()
                    .filter_map(|planet| {
                        planet["name"].as_str().map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let text = fs::read_to_string(&description_path)?;
        let normalized = validate::normalize_article(
            &text,
            spectral_label.as_deref(),
            &planet_names,
        );
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
        let rows = run(&root.0, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["changed"], true);
        let rewritten =
            fs::read_to_string(system.join("description.md")).unwrap();
        assert!(rewritten.contains("**0.51 times Earth's**"));
        let rows = run(&root.0, None).unwrap();
        assert_eq!(rows[0]["changed"], false, "second pass is a no-op");
        let other = root.0.join("no-description");
        fs::create_dir(&other).unwrap();
        let rows = run(&root.0, None).unwrap();
        assert_eq!(rows.len(), 1, "directories without articles are skipped");
    }

    #[test]
    fn normalizes_only_the_selected_variant_without_touching_metadata() {
        let root = TestDirectory::new();
        let system = root.0.join("alpha");
        fs::create_dir(&system).unwrap();
        let original = "# Alpha\n\nIts minimum mass is **about 5 Earth masses**. We know little about its composition.";
        for filename in ["description.md", "description_pass2.md"] {
            fs::write(system.join(filename), original).unwrap();
        }
        fs::write(system.join("metadata_pass2.toml"), "unchanged metadata")
            .unwrap();
        assert!(run(&root.0, Some("../invalid")).is_err());
        let rows = run(&root.0, Some("pass2")).unwrap();
        assert_eq!(rows[0]["changed"], true);
        assert_eq!(
            fs::read_to_string(system.join("description.md")).unwrap(),
            original
        );
        assert_eq!(
            fs::read_to_string(system.join("description_pass2.md")).unwrap(),
            original.replace("about 5", "5")
        );
        assert_eq!(
            fs::read_to_string(system.join("metadata_pass2.toml")).unwrap(),
            "unchanged metadata"
        );
        assert_eq!(run(&root.0, Some("pass2")).unwrap()[0]["changed"], false);
    }
}
