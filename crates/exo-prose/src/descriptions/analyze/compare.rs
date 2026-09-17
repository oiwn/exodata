//! Side-by-side comparison of two corpora (for example a pass-1
//! snapshot and a regeneration), used to evaluate prompt and model
//! changes against the headline analysis metrics. Only hostnames
//! described in both corpora count, so a variant corpus compares
//! against the same systems in the baseline.
use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

use super::{SystemDoc, tropes};

pub fn columns() -> Vec<String> {
    ["metric", "baseline", "candidate", "delta"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line comparison for the `lines` output format.
pub fn line(row: &Value) -> String {
    let metric = row["metric"].as_str().unwrap_or("?");
    let baseline = row["baseline"].as_str().unwrap_or("-");
    let candidate = row["candidate"].as_str().unwrap_or("-");
    let delta = row["delta"].as_str().unwrap_or("");
    format!("{metric:<28} {baseline:>12} {candidate:>12}  {delta}")
}

fn summary_map(docs: &[&SystemDoc]) -> HashMap<String, Value> {
    super::text::summary_docs(docs)
        .into_iter()
        .map(|row| (row["metric"].as_str().unwrap_or_default().to_owned(), row))
        .collect()
}

fn stat(map: &HashMap<String, Value>, metric: &str, key: &str) -> f64 {
    map.get(metric)
        .and_then(|row| row[key].as_f64())
        .unwrap_or_default()
}

fn fmt(value: f64) -> String {
    let formatted = format!("{value:.2}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

fn unbolded_total(docs: &[&SystemDoc]) -> f64 {
    docs.iter()
        .map(|doc| doc.unbolded_measurements())
        .sum::<usize>() as f64
}

fn trope_map(docs: &[&SystemDoc]) -> HashMap<String, Value> {
    tropes::tropes_docs(docs)
        .into_iter()
        .map(|row| (row["trope"].as_str().unwrap_or_default().to_owned(), row))
        .collect()
}

/// Hostnames described in both corpora.
fn shared_systems<'a>(
    baseline: &[&'a SystemDoc],
    candidate: &[&'a SystemDoc],
) -> Vec<&'a SystemDoc> {
    let candidate_names: HashSet<&str> =
        candidate.iter().map(|doc| doc.hostname.as_str()).collect();
    baseline
        .iter()
        .filter(|doc| candidate_names.contains(doc.hostname.as_str()))
        .copied()
        .collect()
}

/// Metric rows comparing distributions, headline counts, and trope
/// reach between the shared systems of two corpora.
pub fn run(baseline: &super::Corpus, candidate: &super::Corpus) -> Vec<Value> {
    let all_baseline: Vec<&SystemDoc> =
        baseline.described().filter(|doc| !doc.failed).collect();
    let all_candidate: Vec<&SystemDoc> =
        candidate.described().filter(|doc| !doc.failed).collect();
    let base = shared_systems(&all_baseline, &all_candidate);
    let cand = shared_systems(&all_candidate, &all_baseline);
    let mut rows = Vec::new();
    let baseline_failed =
        baseline.systems.iter().filter(|doc| doc.failed).count();
    let candidate_failed =
        candidate.systems.iter().filter(|doc| doc.failed).count();
    rows.push(json!({
        "metric": "systems_failed",
        "baseline": baseline_failed.to_string(),
        "candidate": candidate_failed.to_string(),
        "delta": format!("{:+}", candidate_failed as i64 - baseline_failed as i64),
    }));
    let base_summary = summary_map(&base);
    let cand_summary = summary_map(&cand);
    fn push(
        rows: &mut Vec<Value>,
        base: &HashMap<String, Value>,
        cand: &HashMap<String, Value>,
        metric: &str,
        key: &str,
        label: &str,
    ) {
        let b = stat(base, metric, key);
        let c = stat(cand, metric, key);
        rows.push(json!({
            "metric": label,
            "baseline": fmt(b),
            "candidate": fmt(c),
            "delta": format!("{:+}", fmt(c - b)),
        }));
    }
    rows.push(json!({
        "metric": "systems_compared",
        "baseline": base.len().to_string(),
        "candidate": cand.len().to_string(),
        "delta": format!(
            "{:+}",
            cand.len() as i64 - base.len() as i64
        ),
    }));
    push(
        &mut rows,
        &base_summary,
        &cand_summary,
        "words",
        "median",
        "words_median",
    );
    push(
        &mut rows,
        &base_summary,
        &cand_summary,
        "words",
        "mean",
        "words_mean",
    );
    push(
        &mut rows,
        &base_summary,
        &cand_summary,
        "sentences",
        "median",
        "sentences_median",
    );
    push(
        &mut rows,
        &base_summary,
        &cand_summary,
        "sentences",
        "mean",
        "sentences_mean",
    );
    push(
        &mut rows,
        &base_summary,
        &cand_summary,
        "avg_sentence_words",
        "mean",
        "avg_sentence_words_mean",
    );
    push(
        &mut rows,
        &base_summary,
        &cand_summary,
        "ttr",
        "mean",
        "ttr_mean",
    );
    let (b, c) = (unbolded_total(&base), unbolded_total(&cand));
    rows.push(json!({
        "metric": "unbolded_measurements",
        "baseline": fmt(b),
        "candidate": fmt(c),
        "delta": format!("{:+}", fmt(c - b)),
    }));
    let (b, c) = (
        super::cross_system_duplicate_count(&base) as f64,
        super::cross_system_duplicate_count(&cand) as f64,
    );
    rows.push(json!({
        "metric": "duplicate_sentences",
        "baseline": fmt(b),
        "candidate": fmt(c),
        "delta": format!("{:+}", fmt(c - b)),
    }));

    let base_tropes = trope_map(&base);
    let cand_tropes = trope_map(&cand);
    let mut names: Vec<&String> = base_tropes.keys().collect();
    names.sort();
    for name in names {
        let b = base_tropes
            .get(name)
            .and_then(|row| row["systems"].as_u64())
            .unwrap_or_default();
        let c = cand_tropes
            .get(name)
            .and_then(|row| row["systems"].as_u64())
            .unwrap_or_default();
        rows.push(json!({
            "metric": format!("trope_{name}"),
            "baseline": b.to_string(),
            "candidate": c.to_string(),
            "delta": format!("{:+}", c as i64 - b as i64),
        }));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_trims_trailing_zeros() {
        assert_eq!(fmt(148.0), "148");
        assert_eq!(fmt(18.1), "18.1");
    }

    fn doc(hostname: &str, words: &str) -> SystemDoc {
        let (_, paragraphs, sentences) =
            super::super::split_article(&format!("# {hostname}\n\n{words}"));
        SystemDoc {
            hostname: hostname.to_owned(),
            directory: std::path::PathBuf::from("."),
            title: Some(hostname.to_owned()),
            paragraphs,
            sentences,
            metadata: None,
            failed: false,
            described: true,
        }
    }

    #[test]
    fn comparison_uses_only_shared_systems() {
        let baseline = super::super::Corpus {
            systems: vec![
                doc("A", "Alpha prose here. Two sentences total now."),
                doc("B", "Beta prose here. Two sentences also here."),
            ],
        };
        let candidate = super::super::Corpus {
            systems: vec![doc("A", "Alpha variant text. Also two lines.")],
        };
        let rows = run(&baseline, &candidate);
        let systems = rows
            .iter()
            .find(|row| row["metric"] == "systems_compared")
            .unwrap();
        assert_eq!(systems["baseline"], "1");
        assert_eq!(systems["candidate"], "1");
    }

    #[test]
    fn failed_latest_attempts_exclude_old_successes_on_either_side() {
        let mut base_failed = doc("B", "Old baseline success.");
        base_failed.failed = true;
        let mut candidate_failed = doc("C", "Old candidate success.");
        candidate_failed.failed = true;
        let baseline = super::super::Corpus {
            systems: vec![
                doc("A", "Current baseline."),
                base_failed,
                doc("C", "Current C."),
            ],
        };
        let candidate = super::super::Corpus {
            systems: vec![
                doc("A", "Current candidate."),
                doc("B", "Current B."),
                candidate_failed,
            ],
        };
        let rows = run(&baseline, &candidate);
        for metric in ["systems_compared", "systems_failed"] {
            let row = rows.iter().find(|row| row["metric"] == metric).unwrap();
            assert_eq!(row["baseline"], "1");
            assert_eq!(row["candidate"], "1");
        }
    }
}
