//! Curated trope report: known prose warts with counts and affected
//! systems. Frequency-based monotony is covered by n-grams and
//! templates; this table names semantic patterns.
use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Value, json};

pub fn columns() -> Vec<String> {
    ["trope", "count", "systems", "example_systems"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Compact single-line trope row for the `lines` output format.
pub fn line(row: &Value) -> String {
    let trope = row["trope"].as_str().unwrap_or("?");
    let count = row["count"].as_u64().unwrap_or_default();
    let systems = row["systems"].as_u64().unwrap_or_default();
    let examples = row["example_systems"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    format!("{trope:<28} {count:>5}x {systems:>5}s  {examples}")
}

/// Named trope patterns matched against body text (case-insensitive).
const TROPES: &[(&str, &str)] = &[
    ("listed_planets_ranking", "among the listed planets"),
    ("orbit_completion", "completes an orbit"),
    ("earth_year_anchor", "365-day"),
    ("radial_velocity_explainer", "radial velocity method, which"),
    ("associated_one_planet", "one planet is associated"),
    ("associated_with_host", "associated with the host"),
    ("discovered_passive", "was discovered in"),
    ("larger_than_earths", "larger than Earth's"),
    ("smaller_than_earths", "smaller than Earth's"),
    ("larger_than_sun", "larger than the Sun"),
    ("smaller_than_sun", "smaller than the Sun"),
    ("times_earths", "times Earth's"),
    ("times_jupiters", "times Jupiter's"),
    ("hotter_than_sun", "hotter than the Sun"),
    ("cooler_than_sun", "cooler than the Sun"),
    ("sun_like", "Sun-like"),
];

static MATCHERS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    TROPES
        .iter()
        .map(|(name, pattern)| {
            (*name, Regex::new(&format!("(?i){pattern}")).unwrap())
        })
        .collect()
});

/// One row per trope: total matches and affected systems.
pub fn run(corpus: &super::Corpus) -> Vec<Value> {
    tropes_docs(&corpus.described().collect::<Vec<_>>())
}

/// Trope rows over a selected slice of described systems, for
/// corpus-to-corpus comparison.
pub(crate) fn tropes_docs(docs: &[&super::SystemDoc]) -> Vec<Value> {
    MATCHERS
        .iter()
        .map(|(name, regex)| {
            let mut count = 0u64;
            let mut system_count = 0u64;
            let mut examples = Vec::new();
            for doc in docs {
                let matches: u64 = doc
                    .paragraphs
                    .iter()
                    .map(|p| regex.find_iter(p).count() as u64)
                    .sum();
                if matches > 0 {
                    count += matches;
                    system_count += 1;
                    if examples.len() < 5 {
                        examples.push(doc.hostname.clone());
                    }
                }
            }
            json!({
                "trope": name,
                "count": count,
                "systems": system_count,
                "example_systems": examples,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tropes_match_case_insensitively() {
        let (_, regex) = &MATCHERS[0];
        assert!(regex.is_match("ranked among the listed planets."));
        assert!(regex.is_match("Among The Listed Planets!"));
        assert!(!regex.is_match("among planets"));
    }
}
