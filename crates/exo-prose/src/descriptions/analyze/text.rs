//! Per-system text metrics and corpus-level length distributions.
use serde_json::{Value, json};

use super::SystemDoc;

pub fn columns() -> Vec<String> {
    [
        "hostname",
        "words",
        "sentences",
        "paragraphs",
        "avg_sentence_words",
        "ttr",
        "title_words",
        "unbolded_measurements",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// Compact single-line text metrics for the `lines` output format.
pub fn line(row: &Value) -> String {
    let hostname = row["hostname"].as_str().unwrap_or("?");
    let words = row["words"].as_u64().unwrap_or_default();
    let sentences = row["sentences"].as_u64().unwrap_or_default();
    let paragraphs = row["paragraphs"].as_u64().unwrap_or_default();
    let avg = row["avg_sentence_words"].as_f64().unwrap_or_default();
    let ttr = row["ttr"].as_f64().unwrap_or_default();
    let title = row["title_words"].as_u64();
    let unbolded = row["unbolded_measurements"].as_u64().unwrap_or_default();
    format!(
        "{hostname:<26} {words:>5}w {sentences:>3}s {paragraphs:>2}p avg {avg:>5.1}w ttr {ttr:.3} title {} unbolded {unbolded}",
        match title {
            Some(t) => format!("{t}w"),
            None => "-".to_owned(),
        }
    )
}

fn metrics(doc: &SystemDoc) -> (f64, f64, f64, f64, f64, Option<f64>, f64) {
    let words = doc.words();
    let sentences = doc.sentences.len();
    let avg = if sentences == 0 {
        0.0
    } else {
        words as f64 / sentences as f64
    };
    let ttr = if words == 0 {
        0.0
    } else {
        doc.distinct_words() as f64 / words as f64
    };
    (
        words as f64,
        sentences as f64,
        doc.paragraphs.len() as f64,
        avg,
        ttr,
        doc.title_words().map(|t| t as f64),
        doc.unbolded_measurements() as f64,
    )
}

/// One row per described system.
pub fn run(corpus: &super::Corpus) -> Vec<Value> {
    corpus
        .described()
        .map(|doc| {
            let (words, sentences, paragraphs, avg, ttr, title_words, unbolded) =
                metrics(doc);
            json!({
                "hostname": doc.hostname,
                "words": words as u64,
                "sentences": sentences as u64,
                "paragraphs": paragraphs as u64,
                "avg_sentence_words": super::round(avg, 2),
                "ttr": super::round(ttr, 4),
                "title_words": title_words.map(|t| t as u64),
                "unbolded_measurements": unbolded as u64,
            })
        })
        .collect()
}

fn distribution_rows(samples: &[(String, Vec<f64>)]) -> Vec<Value> {
    samples
        .iter()
        .map(|(metric, values)| {
            if values.is_empty() {
                return json!({
                    "metric": metric,
                    "n": 0,
                    "min": Value::Null,
                    "p5": Value::Null,
                    "p25": Value::Null,
                    "median": Value::Null,
                    "mean": Value::Null,
                    "p75": Value::Null,
                    "p95": Value::Null,
                    "max": Value::Null,
                    "stddev": Value::Null,
                });
            }
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.total_cmp(b));
            let (mean, stddev) = super::mean_stddev(values);
            json!({
                "metric": metric,
                "n": values.len(),
                "min": super::round(sorted[0], 2),
                "p5": super::round(super::percentile(&sorted, 5), 2),
                "p25": super::round(super::percentile(&sorted, 25), 2),
                "median": super::round(super::percentile(&sorted, 50), 2),
                "mean": super::round(mean, 2),
                "p75": super::round(super::percentile(&sorted, 75), 2),
                "p95": super::round(super::percentile(&sorted, 95), 2),
                "max": super::round(sorted[sorted.len() - 1], 2),
                "stddev": super::round(stddev, 2),
            })
        })
        .collect()
}

pub fn summary_columns() -> Vec<String> {
    [
        "metric", "n", "min", "p5", "p25", "median", "mean", "p75", "p95", "max",
        "stddev",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// Compact single-line distribution for the `lines` output format.
pub fn summary_line(row: &Value) -> String {
    let metric = row["metric"].as_str().unwrap_or("?");
    let n = row["n"].as_u64().unwrap_or_default();
    let fmt = |key: &str| -> String {
        row[key]
            .as_f64()
            .map(|v| format!("{v:.2}"))
            .unwrap_or_else(|| "-".to_owned())
    };
    format!(
        "{metric:<20} n {n:>4}  min {:>8}  p5 {:>8}  p50 {:>8}  mean {:>8}  p95 {:>8}  max {:>8}",
        fmt("min"),
        fmt("p5"),
        fmt("median"),
        fmt("mean"),
        fmt("p95"),
        fmt("max"),
    )
}

/// Corpus-level distributions over per-system text metrics.
pub fn summary(corpus: &super::Corpus) -> Vec<Value> {
    summary_docs(&corpus.described().collect::<Vec<_>>())
}

/// Distributions over a selected slice of described systems, for
/// corpus-to-corpus comparison.
pub(crate) fn summary_docs(docs: &[&SystemDoc]) -> Vec<Value> {
    let mut words = Vec::new();
    let mut sentences = Vec::new();
    let mut paragraphs = Vec::new();
    let mut avg_sentence = Vec::new();
    let mut ttr = Vec::new();
    let mut title_words = Vec::new();
    let mut unbolded = Vec::new();
    for doc in docs {
        let (w, s, p, avg, ratio, title, unb) = metrics(doc);
        words.push(w);
        sentences.push(s);
        paragraphs.push(p);
        avg_sentence.push(avg);
        ttr.push(ratio);
        if let Some(title) = title {
            title_words.push(title);
        }
        unbolded.push(unb);
    }
    distribution_rows(&[
        ("words".into(), words),
        ("sentences".into(), sentences),
        ("paragraphs".into(), paragraphs),
        ("avg_sentence_words".into(), avg_sentence),
        ("ttr".into(), ttr),
        ("title_words".into(), title_words),
        ("unbolded_measurements".into(), unbolded),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_orders_rows_by_metric() {
        let rows = [
            json!({"metric": "words", "n": 2, "min": 10.0, "median": 15.0,
                   "mean": 15.0, "max": 20.0}),
            json!({"metric": "ttr", "n": 2}),
        ];
        let metrics: Vec<&str> =
            rows.iter().map(|r| r["metric"].as_str().unwrap()).collect();
        assert_eq!(metrics, ["words", "ttr"]);
    }
}
