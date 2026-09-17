//! Machine-readable corpus stats snapshot, written under
//! `content/stats/` (gitignored) as a durable record for before/after
//! comparison across generation passes.
use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use super::{anomalies, meta, ngrams, text, tropes};

pub fn columns() -> Vec<String> {
    ["path", "sections", "systems", "described"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line snapshot summary for the `lines` output format.
pub fn line(row: &Value) -> String {
    let path = row["path"].as_str().unwrap_or("?");
    let sections = row["sections"].as_u64().unwrap_or_default();
    let systems = row["systems"].as_u64().unwrap_or_default();
    let described = row["described"].as_u64().unwrap_or_default();
    format!(
        "{path}: {sections} sections, {systems} systems, {described} described"
    )
}

/// Write one JSON snapshot combining every analysis section.
pub fn run(
    corpus: &super::Corpus,
    corpus_dir: &Path,
    output_path: &Path,
) -> Result<Vec<Value>> {
    let snapshot = json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "corpus": corpus_dir.display().to_string(),
        "systems": corpus.count(),
        "described": corpus.described().count(),
        "failed": corpus.systems.iter().filter(|s| s.failed).count(),
        "summary": text::summary(corpus),
        "tropes": tropes::run(corpus),
        "templates": ngrams::templates(
            corpus,
            &ngrams::TemplateOptions { top: 30, min_count: 5 },
        ),
        "metadata": meta::run(corpus),
        "duplicate_sentences": duplicate_summary(corpus),
        "unbolded_measurements": corpus
            .described()
            .map(|doc| doc.unbolded_measurements())
            .sum::<usize>(),
    });
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create {}", parent.display()))?;
    }
    fs::write(
        output_path,
        serde_json::to_string_pretty(&snapshot)
            .context("Cannot serialize snapshot")?,
    )
    .with_context(|| format!("Cannot write {}", output_path.display()))?;
    Ok(vec![json!({
        "path": output_path.display().to_string(),
        "sections": 6,
        "systems": corpus.count(),
        "described": corpus.described().count(),
    })])
}

/// Distinct cross-system duplicate sentences: total count plus the ten
/// most widespread, with their system lists.
fn duplicate_summary(corpus: &super::Corpus) -> Value {
    let rows = anomalies::run(
        corpus,
        &anomalies::Options {
            z_threshold: f64::INFINITY,
            top: 10,
            min_duplicate_systems: 2,
        },
    );
    let duplicates: Vec<&Value> = rows
        .iter()
        .filter(|row| row["kind"] == "duplicate_sentence")
        .collect();
    json!({
        "distinct": super::cross_system_duplicate_count(
            &corpus.described().collect::<Vec<_>>(),
        ),
        "top": duplicates,
    })
}
