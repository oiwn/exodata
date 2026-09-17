//! Full markdown analysis report combining every section.
use std::fs;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use super::{anomalies, meta, ngrams, text, tropes};

pub fn columns() -> Vec<String> {
    ["path", "sections", "systems", "described"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line report summary for the `lines` output format.
pub fn line(row: &Value) -> String {
    let path = row["path"].as_str().unwrap_or("?");
    let sections = row["sections"].as_u64().unwrap_or_default();
    let systems = row["systems"].as_u64().unwrap_or_default();
    let described = row["described"].as_u64().unwrap_or_default();
    format!(
        "{path}: {sections} sections, {systems} systems, {described} described"
    )
}

fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str("| ");
    out.push_str(&headers.join(" | "));
    out.push_str(" |\n|");
    for _ in headers {
        out.push_str("---|");
    }
    out.push('\n');
    for row in rows {
        out.push_str("| ");
        out.push_str(&row.join(" | "));
        out.push_str(" |\n");
    }
    out
}

fn cell(value: &Value, key: &str) -> String {
    match &value[key] {
        Value::Null => "-".to_owned(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Array(list) => list
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_string(),
    }
}

/// Run every analysis section and write one markdown report.
pub fn run(
    corpus: &super::Corpus,
    output: &std::path::Path,
) -> Result<Vec<Value>> {
    let mut markdown = String::new();
    let mut sections = 0u64;

    markdown.push_str("# Description Analysis Report\n\n");
    markdown.push_str(&format!(
        "- systems with requests: {}\n- described: {}\n- failed: {}\n\n",
        corpus.count(),
        corpus.described().count(),
        corpus.systems.iter().filter(|s| s.failed).count(),
    ));

    markdown.push_str("## Text distributions\n\n");
    let rows = text::summary(corpus);
    markdown.push_str(&table(
        &text::summary_columns()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        &rows
            .iter()
            .map(|row| {
                text::summary_columns()
                    .iter()
                    .map(|column| cell(row, column))
                    .collect()
            })
            .collect::<Vec<_>>(),
    ));
    markdown.push('\n');
    sections += 1;

    markdown.push_str("## Top word n-grams (1-5)\n\n");
    for row in ngrams::run(
        corpus,
        &ngrams::Options {
            top: 10,
            min_n: 1,
            max_n: 5,
            openers: false,
        },
    ) {
        markdown.push_str(&format!(
            "- {}g {}x in {} systems: {}\n",
            cell(&row, "n"),
            cell(&row, "frequency"),
            cell(&row, "documents"),
            cell(&row, "gram")
        ));
    }
    markdown.push('\n');
    sections += 1;

    markdown.push_str("## Sentence openers (1-3 grams)\n\n");
    for row in ngrams::run(
        corpus,
        &ngrams::Options {
            top: 10,
            min_n: 1,
            max_n: 3,
            openers: true,
        },
    ) {
        markdown.push_str(&format!(
            "- {}g {}x in {} systems: {}\n",
            cell(&row, "n"),
            cell(&row, "frequency"),
            cell(&row, "documents"),
            cell(&row, "gram")
        ));
    }
    markdown.push('\n');
    sections += 1;

    markdown.push_str("## Masked sentence templates\n\n");
    for row in ngrams::templates(
        corpus,
        &ngrams::TemplateOptions {
            top: 20,
            min_count: 5,
        },
    ) {
        markdown.push_str(&format!(
            "- {}x in {} systems: {}\n",
            cell(&row, "frequency"),
            cell(&row, "documents"),
            cell(&row, "template")
        ));
    }
    markdown.push('\n');
    sections += 1;

    markdown.push_str("## Trope report\n\n");
    markdown.push_str(&table(
        &tropes::columns()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        &tropes::run(corpus)
            .iter()
            .map(|row| {
                tropes::columns()
                    .iter()
                    .map(|column| cell(row, column))
                    .collect()
            })
            .collect::<Vec<_>>(),
    ));
    markdown.push('\n');
    sections += 1;

    markdown.push_str("## Generation metadata\n\n");
    markdown.push_str(&table(
        &meta::columns()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        &meta::run(corpus)
            .iter()
            .map(|row| {
                meta::columns()
                    .iter()
                    .map(|column| cell(row, column))
                    .collect()
            })
            .collect::<Vec<_>>(),
    ));
    markdown.push('\n');
    sections += 1;

    markdown.push_str("## Anomalies (top 30)\n\n");
    markdown.push_str(&table(
        &anomalies::columns()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        &anomalies::run(
            corpus,
            &anomalies::Options {
                z_threshold: 2.5,
                top: 30,
                min_duplicate_systems: 2,
            },
        )
        .iter()
        .map(|row| {
            anomalies::columns()
                .iter()
                .map(|column| cell(row, column))
                .collect()
        })
        .collect::<Vec<_>>(),
    ));
    markdown.push('\n');
    sections += 1;

    fs::write(output, markdown)
        .with_context(|| format!("Cannot write {}", output.display()))?;

    Ok(vec![json!({
        "path": output.display().to_string(),
        "sections": sections,
        "systems": corpus.count(),
        "described": corpus.described().count(),
    })])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_renders_headers_and_rows() {
        let rendered = table(&["a", "b"], &[vec!["1".into(), "2".into()]]);
        assert!(rendered.starts_with("| a | b |\n|---|---|\n"));
        assert!(rendered.ends_with("| 1 | 2 |\n"));
    }
}
