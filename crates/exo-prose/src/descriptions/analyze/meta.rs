//! Generation metadata aggregation: tokens by stage, attempts, models,
//! dates, critic activity, and validation recoveries.
use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::{percentile, round};

pub fn columns() -> Vec<String> {
    ["metric", "value", "detail"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line metric for the `lines` output format.
pub fn line(row: &Value) -> String {
    let metric = row["metric"].as_str().unwrap_or("?");
    let value = row["value"].as_str().unwrap_or("-");
    let detail = row["detail"].as_str().unwrap_or("");
    format!("{metric:<24} {value:>14}  {detail}")
}

fn token_rows(rows: &mut Vec<Value>, metric: &str, values: &[u64], unit: &str) {
    if values.is_empty() {
        rows.push(json!({"metric": metric, "value": "-", "detail": "no data"}));
        return;
    }
    let floats: Vec<f64> = values.iter().map(|v| *v as f64).collect();
    let mut sorted = floats.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let mean = floats.iter().sum::<f64>() / floats.len() as f64;
    rows.push(json!({
        "metric": metric,
        "value": values.iter().sum::<u64>().to_string(),
        "detail": format!(
            "mean {:.0} {} / system, median {:.0}, max {:.0}, n {}",
            round(mean, 0),
            unit,
            percentile(&sorted, 50),
            sorted[sorted.len() - 1],
            values.len(),
        ),
    }));
}

/// Aggregate every `metadata.toml` in the corpus into metric rows.
pub fn run(corpus: &super::Corpus) -> Vec<Value> {
    let mut rows = Vec::new();
    rows.push(json!({
        "metric": "systems",
        "value": corpus.count().to_string(),
        "detail": format!(
            "{} described, {} failed, {} undescribed",
            corpus.described().count(),
            corpus.systems.iter().filter(|s| s.failed).count(),
            corpus.systems.iter().filter(|s| !s.described).count(),
        ),
    }));

    let metadata: Vec<&Value> = corpus
        .systems
        .iter()
        .filter_map(|s| s.metadata.as_ref())
        .collect();

    let get = |key: &str| -> Vec<u64> {
        metadata.iter().filter_map(|m| m[key].as_u64()).collect()
    };
    rows.push(json!({"metric": "usage_incomplete", "value": metadata.iter().filter(|m| m["usage_complete"].as_bool() == Some(false)).count().to_string(), "detail": "systems with missing usage; totals include only known tokens"}));
    token_rows(&mut rows, "tokens_total", &get("total_tokens"), "tokens");
    token_rows(&mut rows, "tokens_prompt", &get("prompt_tokens"), "tokens");
    token_rows(
        &mut rows,
        "tokens_completion",
        &get("completion_tokens"),
        "tokens",
    );
    token_rows(&mut rows, "elapsed_ms", &get("elapsed_ms"), "ms");

    let stage = |name: &str, key: &str| -> Vec<u64> {
        metadata
            .iter()
            .filter_map(|m| m["stages"][name][key].as_u64())
            .collect()
    };
    token_rows(
        &mut rows,
        "stage_draft_prompt",
        &stage("draft", "prompt_tokens"),
        "tokens",
    );
    token_rows(
        &mut rows,
        "stage_draft_completion",
        &stage("draft", "completion_tokens"),
        "tokens",
    );
    token_rows(
        &mut rows,
        "stage_style_prompt",
        &stage("style", "prompt_tokens"),
        "tokens",
    );
    token_rows(
        &mut rows,
        "stage_style_completion",
        &stage("style", "completion_tokens"),
        "tokens",
    );

    let attempts = get("attempts");
    token_rows(
        &mut rows,
        "stage_repair_prompt",
        &stage("repair", "prompt_tokens"),
        "tokens",
    );
    token_rows(
        &mut rows,
        "stage_repair_completion",
        &stage("repair", "completion_tokens"),
        "tokens",
    );
    if !attempts.is_empty() {
        let mut histogram: BTreeMap<u64, u64> = BTreeMap::new();
        for value in &attempts {
            *histogram.entry(*value).or_default() += 1;
        }
        let floats: Vec<f64> = attempts.iter().map(|v| *v as f64).collect();
        let mut sorted = floats.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        rows.push(json!({
            "metric": "attempts",
            "value": format!(
                "median {:.1}",
                percentile(&sorted, 50)
            ),
            "detail": histogram
                .iter()
                .map(|(attempts, systems)| format!("{attempts} att: {systems}"))
                .collect::<Vec<_>>()
                .join(", "),
        }));
    }

    let mut models: BTreeMap<String, u64> = BTreeMap::new();
    let mut fingerprints: BTreeMap<String, u64> = BTreeMap::new();
    let mut dates: Vec<&str> = Vec::new();
    let mut recovered_systems = 0u64;
    let mut recovered_findings = 0u64;
    for m in &metadata {
        if let Some(model) = m["model"].as_str() {
            *models.entry(model.to_owned()).or_default() += 1;
        }
        if let Some(version) = m["fingerprint_version"].as_u64() {
            *fingerprints.entry(version.to_string()).or_default() += 1;
        }
        if let Some(date) = m["generated_at"].as_str() {
            dates.push(date);
        }
        if let Some(list) = m["validation_recovered"]
            .as_array()
            .filter(|list| !list.is_empty())
        {
            recovered_systems += 1;
            recovered_findings += list.len() as u64;
        }
    }
    rows.push(json!({
        "metric": "served_models",
        "value": models.len().to_string(),
        "detail": models
            .iter()
            .map(|(model, systems)| format!("{model}: {systems}"))
            .collect::<Vec<_>>()
            .join(", "),
    }));
    rows.push(json!({
        "metric": "fingerprint_versions",
        "value": fingerprints.len().to_string(),
        "detail": fingerprints
            .iter()
            .map(|(version, systems)| format!("fp{version}: {systems}"))
            .collect::<Vec<_>>()
            .join(", "),
    }));
    if !dates.is_empty() {
        dates.sort_unstable();
        rows.push(json!({
            "metric": "generated_at_range",
            "value": dates.len().to_string(),
            "detail": format!("{} .. {}", dates[0], dates[dates.len() - 1]),
        }));
    }
    rows.push(json!({
        "metric": "validation_recovered",
        "value": recovered_findings.to_string(),
        "detail": format!("{recovered_systems} systems with recovered violations"),
    }));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_renders_metric_value_detail() {
        let row = json!({
            "metric": "tokens_total",
            "value": "39900",
            "detail": "mean 8373 tokens / system",
        });
        let text = line(&row);
        assert!(text.contains("tokens_total"));
        assert!(text.contains("39900"));
    }
}
