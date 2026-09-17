//! N-gram frequencies and masked sentence templates.
use std::collections::HashMap;

use serde_json::{Value, json};

use super::{Counter, sentence_tokens};

pub fn columns() -> Vec<String> {
    ["n", "gram", "frequency", "documents", "doc_share"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line n-gram row for the `lines` output format.
pub fn line(row: &Value) -> String {
    let n = row["n"].as_u64().unwrap_or_default();
    let frequency = row["frequency"].as_u64().unwrap_or_default();
    let documents = row["documents"].as_u64().unwrap_or_default();
    let share = row["doc_share"].as_f64().unwrap_or_default();
    let gram = row["gram"].as_str().unwrap_or("?");
    format!("{n}g {frequency:>7}x {documents:>5}d {share:>5.1}%  {gram}")
}

pub struct Options {
    pub top: usize,
    pub min_n: usize,
    pub max_n: usize,
    pub openers: bool,
}

/// Word n-grams (1-5) with corpus frequency and document frequency.
/// With `openers`, only the first `n` tokens of each sentence count, the
/// template-monotony detector.
pub fn run(corpus: &super::Corpus, options: &Options) -> Vec<Value> {
    let total_docs = corpus.described().count().max(1) as f64;
    let mut counters: Vec<Counter> = (options.min_n..=options.max_n)
        .map(|_| Counter::new())
        .collect();
    for doc in corpus.described() {
        let mut local: Vec<HashMap<String, u64>> = (options.min_n
            ..=options.max_n)
            .map(|_| HashMap::new())
            .collect();
        for sentence in &doc.sentences {
            let tokens = sentence_tokens(sentence);
            for (index, n) in (options.min_n..=options.max_n).enumerate() {
                if tokens.len() < n {
                    continue;
                }
                if options.openers {
                    let gram = tokens[..n].join(" ");
                    *local[index].entry(gram).or_default() += 1;
                } else {
                    for gram in tokens.windows(n).map(|w| w.join(" ")) {
                        *local[index].entry(gram).or_default() += 1;
                    }
                }
            }
        }
        for (index, counter) in counters.iter_mut().enumerate() {
            counter.merge(&local[index]);
        }
    }
    let mut rows = Vec::new();
    for (index, counter) in counters.iter().enumerate() {
        for (gram, frequency, documents) in counter.top(options.top) {
            rows.push(json!({
                "n": options.min_n + index,
                "gram": gram,
                "frequency": frequency,
                "documents": documents,
                "doc_share": super::round(documents as f64 / total_docs * 100.0, 2),
            }));
        }
    }
    rows
}

/// Words masked to `<unit>` when they directly follow a number.
pub(super) const UNITS: &[&str] = &[
    "day", "days", "year", "years", "hour", "hours", "minute", "minutes",
    "second", "seconds", "earth", "earths", "jupiter", "jupiters", "sun", "suns",
    "solar", "au", "pc", "k", "kelvin", "kelvins", "light", "mass", "masses",
    "radii", "radius", "billion", "million", "thousand", "g",
];

/// Mask a normalized sentence for template mining: numbers become
/// `<num>` and up to two unit-lexicon words directly after a number
/// become `<unit>`. Unit words elsewhere ("its mass is ...") stay
/// literal. Numbers inside object names ("51 Peg") are masked too, so
/// same-shape names share templates.
pub fn mask_sentence(sentence: &str) -> String {
    let tokens = super::tokenize(&super::plain_text(sentence));
    let mut masked = Vec::with_capacity(tokens.len());
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind == super::TokenKind::Number {
            masked.push("<num>".to_owned());
            index += 1;
            let mut unit_run = 0;
            while index < tokens.len() && unit_run < 2 {
                let candidate = tokens[index].text.to_lowercase();
                if UNITS.contains(&candidate.as_str()) {
                    masked.push("<unit>".to_owned());
                    index += 1;
                    unit_run += 1;
                } else {
                    break;
                }
            }
        } else {
            masked.push(token.text.to_lowercase());
            index += 1;
        }
    }
    masked.join(" ")
}

pub fn template_columns() -> Vec<String> {
    ["template", "frequency", "documents", "example_systems"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line template row for the `lines` output format.
pub fn template_line(row: &Value) -> String {
    let frequency = row["frequency"].as_u64().unwrap_or_default();
    let documents = row["documents"].as_u64().unwrap_or_default();
    let template = row["template"].as_str().unwrap_or("?");
    format!("{frequency:>6}x {documents:>5}d  {template}")
}

pub struct TemplateOptions {
    pub top: usize,
    pub min_count: u64,
}

/// Sentence templates: sentences with numbers and units masked, counted
/// across the corpus. Frequent templates expose boilerplate phrasing.
pub fn templates(
    corpus: &super::Corpus,
    options: &TemplateOptions,
) -> Vec<Value> {
    let mut counter = Counter::new();
    let mut examples: HashMap<String, Vec<String>> = HashMap::new();
    for doc in corpus.described() {
        let mut local = HashMap::new();
        for sentence in &doc.sentences {
            let masked = mask_sentence(sentence);
            if masked.is_empty() {
                continue;
            }
            *local.entry(masked).or_default() += 1;
        }
        for template in local.keys() {
            let list = examples.entry(template.clone()).or_default();
            if list.len() < 5 {
                list.push(doc.hostname.clone());
            }
        }
        counter.merge(&local);
    }
    let mut rows: Vec<Value> = counter
        .top(usize::MAX)
        .into_iter()
        .filter(|(_, frequency, documents)| {
            *frequency >= options.min_count && *documents >= 2
        })
        .take(options.top)
        .map(|(template, frequency, documents)| {
            let systems = examples
                .get(&template)
                .map(|list| list.as_slice())
                .unwrap_or(&[]);
            json!({
                "template": template,
                "frequency": frequency,
                "documents": documents,
                "example_systems": systems,
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        b["frequency"]
            .as_u64()
            .unwrap_or_default()
            .cmp(&a["frequency"].as_u64().unwrap_or_default())
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masking_replaces_numbers_and_units() {
        let masked = mask_sentence(
            "Its mass is **about 194 Earth masses**, or **about 4.23 days**.",
        );
        assert_eq!(
            masked,
            "its mass is about <num> <unit> <unit> or about <num> <unit>"
        );
    }

    #[test]
    fn masking_keeps_units_without_numbers() {
        let masked = mask_sentence("The mass of the star is unknown here.");
        assert_eq!(masked, "the mass of the star is unknown here");
    }

    #[test]
    fn masking_merges_light_year_phrases() {
        let masked = mask_sentence("It lies **about 50.4 light-years** away.");
        assert_eq!(masked, "it lies about <num> <unit> <unit> away");
    }

    #[test]
    fn masking_numbers_in_names_share_templates() {
        let masked = mask_sentence("**51 Peg b** orbits the star.");
        assert_eq!(masked, "<num> peg b orbits the star");
    }
}
