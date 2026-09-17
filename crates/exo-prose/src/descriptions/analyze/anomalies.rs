//! Anomaly detection: distribution outliers, cross-system duplicate
//! sentences, and within-article repetition.
use std::collections::HashMap;

use serde_json::{Value, json};

use super::{normalize_sentence, round};

pub fn columns() -> Vec<String> {
    ["kind", "hostname", "score", "detail"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line anomaly for the `lines` output format.
pub fn line(row: &Value) -> String {
    let kind = row["kind"].as_str().unwrap_or("?");
    let hostname = row["hostname"].as_str().unwrap_or("-");
    let score = row["score"]
        .as_f64()
        .map(|v| format!("{v:>6.2}"))
        .unwrap_or_else(|| "     -".to_owned());
    let detail = row["detail"].as_str().unwrap_or("");
    format!("{kind:<22} {score}  {hostname:<24} {detail}")
}

pub struct Options {
    pub z_threshold: f64,
    pub top: usize,
    pub min_duplicate_systems: u64,
}

fn zscore_rows(
    kind_high: &str,
    kind_low: &str,
    label: &str,
    values: &[(String, f64)],
    threshold: f64,
) -> Vec<Value> {
    let raw: Vec<f64> = values.iter().map(|(_, v)| *v).collect();
    if raw.len() < 2 {
        return Vec::new();
    }
    let (mean, stddev) = super::mean_stddev(&raw);
    if stddev <= f64::EPSILON {
        return Vec::new();
    }
    let mut rows: Vec<Value> = values
        .iter()
        .map(|(hostname, value)| {
            let z = (value - mean) / stddev;
            let (kind, z) = if z >= 0.0 {
                (kind_high, z)
            } else {
                (kind_low, -z)
            };
            json!({
                "kind": kind,
                "hostname": hostname,
                "score": round(z, 2),
                "detail": format!("{label} {value:.1}, mean {mean:.1}"),
            })
        })
        .filter(|row| row["score"].as_f64().unwrap_or_default() >= threshold)
        .collect();
    rows.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&a["score"].as_f64().unwrap_or_default())
    });
    rows
}

/// Outliers, duplicate sentences across systems, repeated sentences
/// within one article, and undescribed systems.
pub fn run(corpus: &super::Corpus, options: &Options) -> Vec<Value> {
    let mut rows = Vec::new();

    let described: Vec<&super::SystemDoc> = corpus.described().collect();
    let words: Vec<(String, f64)> = described
        .iter()
        .map(|d| (d.hostname.clone(), d.words() as f64))
        .collect();
    let sentences: Vec<(String, f64)> = described
        .iter()
        .map(|d| (d.hostname.clone(), d.sentences.len() as f64))
        .collect();
    let avg_sentence: Vec<(String, f64)> = described
        .iter()
        .map(|d| {
            let words = d.words() as f64;
            let count = d.sentences.len().max(1) as f64;
            (d.hostname.clone(), words / count)
        })
        .collect();
    let numeric_density: Vec<(String, f64)> = described
        .iter()
        .map(|d| {
            let numbers = d.numbers() as f64;
            let count = d.sentences.len().max(1) as f64;
            (d.hostname.clone(), numbers / count)
        })
        .collect();
    let tokens: Vec<(String, f64)> = described
        .iter()
        .map(|d| {
            (
                d.hostname.clone(),
                d.metadata
                    .as_ref()
                    .and_then(|m| m["total_tokens"].as_u64())
                    .unwrap_or_default() as f64,
            )
        })
        .collect();

    rows.extend(zscore_rows(
        "long_article",
        "short_article",
        "words",
        &words,
        options.z_threshold,
    ));
    rows.extend(zscore_rows(
        "many_sentences",
        "few_sentences",
        "sentences",
        &sentences,
        options.z_threshold,
    ));
    rows.extend(zscore_rows(
        "long_sentences",
        "short_sentences",
        "avg words/sentence",
        &avg_sentence,
        options.z_threshold,
    ));
    rows.extend(zscore_rows(
        "numeric_dense",
        "numeric_sparse",
        "numbers/sentence",
        &numeric_density,
        options.z_threshold,
    ));
    rows.extend(zscore_rows(
        "tokens_high",
        "tokens_low",
        "recorded tokens",
        &tokens,
        options.z_threshold,
    ));

    // Cross-system duplicate sentences.
    let mut occurrences: HashMap<String, Vec<String>> = HashMap::new();
    for doc in &described {
        let mut seen = std::collections::BTreeSet::new();
        for sentence in &doc.sentences {
            let normalized = normalize_sentence(sentence);
            if normalized.len() < 20 {
                continue;
            }
            seen.insert(normalized);
        }
        for normalized in seen {
            occurrences
                .entry(normalized)
                .or_default()
                .push(doc.hostname.clone());
        }
    }
    let mut duplicates: Vec<Value> = occurrences
        .into_iter()
        .filter(|(_, systems)| {
            systems.len() as u64 >= options.min_duplicate_systems
        })
        .map(|(sentence, systems)| {
            let count = systems.len() as u64;
            json!({
                "kind": "duplicate_sentence",
                "hostname": format!("{} systems", count),
                "score": count as f64,
                "detail": format!(
                    "{}; e.g. {}",
                    truncate(&sentence, 70),
                    systems.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                ),
            })
        })
        .collect();
    duplicates.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&a["score"].as_f64().unwrap_or_default())
    });
    rows.extend(duplicates.into_iter().take(options.top));

    // Repeated sentences within one article.
    let mut repeated: Vec<Value> = described
        .iter()
        .filter_map(|doc| {
            let mut counts: HashMap<String, u64> = HashMap::new();
            for sentence in &doc.sentences {
                let normalized = normalize_sentence(sentence);
                if normalized.len() < 20 {
                    continue;
                }
                *counts.entry(normalized).or_default() += 1;
            }
            let (sentence, count) = counts
                .into_iter()
                .filter(|(_, count)| *count >= 2)
                .max_by_key(|(_, count)| *count)?;
            Some(json!({
                "kind": "repeated_sentence",
                "hostname": doc.hostname,
                "score": count as f64,
                "detail": truncate(&sentence, 80),
            }))
        })
        .collect();
    repeated.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&a["score"].as_f64().unwrap_or_default())
    });
    rows.extend(repeated.into_iter().take(options.top));

    // Numbers that read as measurements but sit outside bold spans.
    let mut unbolded: Vec<Value> = described
        .iter()
        .filter_map(|doc| {
            let phrases = doc.unbolded_measurement_phrases();
            (!phrases.is_empty()).then(|| {
                let count = phrases.len();
                let shown: Vec<&str> =
                    phrases.iter().take(3).map(String::as_str).collect();
                json!({
                    "kind": "unbolded_measurement",
                    "hostname": doc.hostname,
                    "score": count as f64,
                    "detail": format!(
                        "{} outside bold spans: {}",
                        if count == 1 {
                            "1 measurement number".to_owned()
                        } else {
                            format!("{count} measurement numbers")
                        },
                        shown.join(", "),
                    ),
                })
            })
        })
        .collect();
    unbolded.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&a["score"].as_f64().unwrap_or_default())
    });
    rows.extend(unbolded.into_iter().take(options.top));

    // Systems with a request but no description.
    rows.extend(corpus.systems.iter().filter(|s| !s.described).map(|s| {
        json!({
            "kind": "missing_description",
            "hostname": s.hostname,
            "score": Value::Null,
            "detail": if s.failed {
                "fail.toml present; no successful generation"
            } else {
                "description.md absent or empty"
            },
        })
    }));

    rows
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut truncated: String = text.chars().take(limit).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_appends_ellipsis() {
        assert_eq!(truncate("abcdef", 3), "abc…");
        assert_eq!(truncate("ab", 3), "ab");
    }
}
