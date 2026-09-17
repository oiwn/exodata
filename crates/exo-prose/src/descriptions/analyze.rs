//! Corpus loading and text primitives for description analysis.
//!
//! Everything here is read-only over the local content directory: the
//! per-system `request.toml` (hostname), `description.md` (title, body),
//! `metadata.toml` (usage and stage data), and `fail.toml` presence.
pub mod anomalies;
pub mod compare;
pub mod meta;
pub mod ngrams;
pub mod report;
pub mod snapshot;
pub mod text;
pub mod tropes;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{Value, json};

/// One token: a number (kept whole across `.` and `,`), a word (with
/// inner apostrophes), or another alphanumeric run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Number,
    Word,
    Other,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

static TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\d+(?:[.,]\d+)*|[A-Za-z]+(?:['’][A-Za-z]+)*").unwrap()
});

/// Split text into tokens; markdown emphasis markers are not tokens.
pub fn tokenize(text: &str) -> Vec<Token> {
    TOKEN
        .find_iter(text)
        .map(|m| {
            let text = m.as_str();
            let kind = if text.as_bytes()[0].is_ascii_digit() {
                TokenKind::Number
            } else if text.as_bytes()[0].is_ascii_alphabetic() {
                TokenKind::Word
            } else {
                TokenKind::Other
            };
            Token {
                text: text.to_owned(),
                kind,
            }
        })
        .collect()
}

/// Split body text into paragraph strings (blank-line separated).
pub fn paragraphs(body: &str) -> Vec<String> {
    body.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Split one paragraph into sentences on sentence-ending punctuation.
/// A period between two digits is a decimal separator, not a sentence
/// end; newlines inside a paragraph are preserved as whitespace.
pub fn split_sentences(paragraph: &str) -> Vec<String> {
    let chars: Vec<char> = paragraph.chars().collect();
    let mut sentences = Vec::new();
    let mut current = String::new();
    for (index, c) in chars.iter().copied().enumerate() {
        current.push(c);
        if matches!(c, '.' | '!' | '?') {
            let decimal = c == '.'
                && index > 0
                && chars[index - 1].is_ascii_digit()
                && chars
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_digit());
            if !decimal {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_owned());
                }
                current.clear();
            }
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_owned());
    }
    sentences
}

/// One analyzed system.
pub struct SystemDoc {
    pub hostname: String,
    pub directory: PathBuf,
    pub title: Option<String>,
    /// Body paragraphs with Markdown emphasis markers preserved (for
    /// bold-span analysis); the title line is excluded.
    pub paragraphs: Vec<String>,
    pub sentences: Vec<String>,
    pub metadata: Option<Value>,
    pub failed: bool,
    pub described: bool,
}

impl SystemDoc {
    /// Body text tokens.
    pub fn tokens(&self) -> Vec<Token> {
        self.paragraphs.iter().flat_map(|p| tokenize(p)).collect()
    }

    /// Tokens of the raw body with a flag marking tokens inside
    /// Markdown strong (`**`) spans. Segments between `**` markers
    /// alternate outside/inside; an unmatched marker treats the tail
    /// as outside.
    pub fn bold_aware_tokens(&self) -> Vec<(Token, bool)> {
        let mut tokens = Vec::new();
        for paragraph in &self.paragraphs {
            for (index, segment) in paragraph.split("**").enumerate() {
                let in_strong = index % 2 == 1;
                for token in tokenize(segment) {
                    tokens.push((token, in_strong));
                }
            }
        }
        tokens
    }

    /// Numbers outside bold spans that read as measurements: a number
    /// followed by a unit-lexicon word, or `about <number>`. Discovery
    /// years (1900-2099) do not count. These are candidates the
    /// algorithmic bolder should have wrapped.
    /// Numbers outside bold spans that read as measurements: a number
    /// followed by a unit-lexicon word, or `about <number>`. Discovery
    /// years (1900-2099) and the 365-day year anchor do not count.
    /// These are candidates the algorithmic bolder should have wrapped.
    pub fn unbolded_measurements(&self) -> usize {
        self.unbolded_measurement_phrases().len()
    }

    /// The flagged phrases themselves, oldest first, as `number unit`
    /// pairs (or `about number` when only the hedge precedes).
    pub fn unbolded_measurement_phrases(&self) -> Vec<String> {
        let tokens = self.bold_aware_tokens();
        let mut phrases = Vec::new();
        for (index, (token, in_strong)) in tokens.iter().enumerate() {
            if *in_strong || token.kind != TokenKind::Number {
                continue;
            }
            if is_discovery_year(&token.text) {
                continue;
            }
            // The guide's Earth-year constant ("Earth's roughly
            // 365-day year") is editorial text, deliberately
            // unbolded and tracked separately by the trope report.
            let next = tokens
                .get(index + 1)
                .map(|(next, _)| next.text.to_lowercase());
            if token.text == "365"
                && matches!(next.as_deref(), Some("day" | "days"))
            {
                continue;
            }
            let unit_follows = next
                .as_deref()
                .is_some_and(|next| ngrams::UNITS.contains(&next));
            // `about <number>` alone also appears in guide-constant
            // restatements ("extends from about 1.5 to 2 Earth radii"),
            // so the hedge only counts when a measurement continues:
            // a unit or a `times` comparison.
            let times_follows = next.as_deref() == Some("times");
            let about_precedes = index > 0
                && tokens[index - 1].0.text.eq_ignore_ascii_case("about")
                && (unit_follows || times_follows);
            if unit_follows || about_precedes {
                phrases.push(match (&next, unit_follows) {
                    (Some(next), true) => {
                        format!("{} {}", token.text, next)
                    }
                    _ => format!("about {}", token.text),
                });
            }
        }
        phrases
    }

    pub fn words(&self) -> usize {
        self.paragraphs
            .iter()
            .map(|p| tokenize(p).len())
            .sum::<usize>()
    }

    pub fn distinct_words(&self) -> usize {
        self.paragraphs
            .iter()
            .flat_map(|p| tokenize(p))
            .map(|t| t.text.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    pub fn title_words(&self) -> Option<usize> {
        self.title.as_ref().map(|t| tokenize(t).len())
    }

    pub fn numbers(&self) -> usize {
        self.paragraphs
            .iter()
            .map(|p| {
                tokenize(p)
                    .into_iter()
                    .filter(|t| t.kind == TokenKind::Number)
                    .count()
            })
            .sum::<usize>()
    }
}

/// Strip markdown emphasis markers and list/heading prefixes from text.
pub fn plain_text(text: &str) -> String {
    text.replace("**", "")
}

/// Four-digit years in the discovery-year range.
fn is_discovery_year(token: &str) -> bool {
    token.len() == 4
        && token.bytes().all(|b| b.is_ascii_digit())
        && (1900..=2099).contains(&token.parse::<u16>().unwrap_or(0))
}

fn read_toml(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    Ok(serde_json::to_value(
        toml::from_str::<toml::Value>(&text)
            .with_context(|| format!("Invalid TOML in {}", path.display()))?,
    )?)
}

/// A loaded content directory: every system directory that holds a
/// `request.toml`, sorted by hostname.
pub struct Corpus {
    pub systems: Vec<SystemDoc>,
}

impl Corpus {
    /// Load a corpus; with a label, `description_<label>.md` and
    /// `metadata_<label>.toml` are read instead of the served files,
    /// for variant analysis.
    pub fn load(content_dir: &Path, label: Option<&str>) -> Result<Self> {
        if let Some(label) = label {
            super::batch::validate_label(label)?;
        }
        let description_file = match label {
            Some(label) => format!("description_{label}.md"),
            None => "description.md".to_owned(),
        };
        let metadata_file = match label {
            Some(label) => format!("metadata_{label}.toml"),
            None => "metadata.toml".to_owned(),
        };
        let failure_file = match label {
            Some(label) => format!("fail_{label}.toml"),
            None => "fail.toml".to_owned(),
        };
        let mut systems = Vec::new();
        for entry in fs::read_dir(content_dir)
            .with_context(|| format!("Cannot read {}", content_dir.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let directory = entry.path();
            if !directory.join("request.toml").try_exists()? {
                continue;
            }
            let request = read_toml(&directory.join("request.toml"))
                .with_context(|| {
                    format!("Invalid request in {}", directory.display())
                })?;
            let hostname = request["system"]["hostname"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    directory
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
            let raw = directory.join(&description_file);
            let description = fs::read_to_string(&raw)
                .ok()
                .filter(|text| !text.trim().is_empty());
            let described = description.is_some();
            let (title, paragraphs, sentences) = description
                .map(|text| split_article(&text))
                .unwrap_or_default();
            let failed = directory.join(&failure_file).try_exists()?;
            let metadata_path = directory.join(if failed {
                &failure_file
            } else {
                &metadata_file
            });
            let mut metadata = if metadata_path.try_exists()? {
                Some(read_toml(&metadata_path)?)
            } else {
                None
            };
            if let Some(record) = &mut metadata {
                super::usage::record_totals(record).write(record);
            }
            systems.push(SystemDoc {
                hostname,
                directory,
                title,
                paragraphs,
                sentences,
                metadata,
                failed,
                described,
            });
        }
        systems.sort_by(|a, b| a.hostname.cmp(&b.hostname));
        Ok(Self { systems })
    }

    pub fn described(&self) -> impl Iterator<Item = &SystemDoc> {
        self.systems.iter().filter(|s| s.described)
    }

    pub fn count(&self) -> usize {
        self.systems.len()
    }
}

/// Split an article into title, body paragraphs, and sentences. Markdown
/// emphasis markers are removed from the body; the title line is
/// excluded from paragraph and sentence statistics.
/// Split an article into title, body paragraphs, and sentences.
/// Markdown emphasis markers are preserved in the body so bold-span
/// analysis can inspect them; token-based statistics ignore the
/// markers. The title line is excluded from paragraph and sentence
/// statistics.
pub fn split_article(text: &str) -> (Option<String>, Vec<String>, Vec<String>) {
    let mut lines = text.lines();
    let title = lines.next().and_then(|first| {
        first
            .strip_prefix("# ")
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
    });
    let consumed = if title.is_some() { 1 } else { 0 };
    let body = text.lines().skip(consumed).collect::<Vec<_>>().join("\n");
    let paragraphs = paragraphs(&body);
    let sentences = paragraphs.iter().flat_map(|p| split_sentences(p)).collect();
    (title, paragraphs, sentences)
}

/// Nearest-rank percentile of a nonempty sorted slice (0-100).
pub fn percentile(sorted: &[f64], p: u32) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p as usize * sorted.len())
        .div_ceil(100)
        .clamp(1, sorted.len());
    sorted[rank - 1]
}

/// Population standard deviation; 0.0 when all values match.
pub fn stddev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64)
        .sqrt()
}

/// Round to `digits` fractional digits for JSON output.
pub fn round(value: f64, digits: u32) -> f64 {
    let factor = 10f64.powi(digits as i32);
    (value * factor).round() / factor
}

/// Mean and standard deviation of a slice.
pub fn mean_stddev(values: &[f64]) -> (f64, f64) {
    let mean = values.iter().sum::<f64>() / values.len().max(1) as f64;
    (mean, stddev(values, mean))
}

/// Count `value` occurrences per key with per-document frequency and
/// document frequency combined in one pass per document.
pub struct Counter {
    counts: HashMap<String, (u64, u64)>,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Merge one document's local counts: `frequency` accumulates totals,
    /// `documents` counts documents mentioning the key at least once.
    pub fn merge(&mut self, local: &HashMap<String, u64>) {
        for (key, count) in local {
            let entry = self.counts.entry(key.clone()).or_insert((0, 0));
            entry.0 += *count;
            entry.1 += 1;
        }
    }

    /// Top `(key, frequency, documents)` pairs by frequency.
    pub fn top(&self, limit: usize) -> Vec<(String, u64, u64)> {
        let mut rows: Vec<_> = self
            .counts
            .iter()
            .map(|(key, (frequency, documents))| {
                (key.clone(), *frequency, *documents)
            })
            .collect();
        rows.sort_by(|a, b| {
            b.1.cmp(&a.1).then(b.2.cmp(&a.2)).then(a.0.cmp(&b.0))
        });
        rows.truncate(limit);
        rows
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Sentence tokens, lowercased, without punctuation.
pub fn sentence_tokens(sentence: &str) -> Vec<String> {
    tokenize(&plain_text(sentence))
        .into_iter()
        .map(|t| t.text.to_lowercase())
        .collect()
}

/// Normalized sentence for duplicate detection: lowercase, single
/// spaces, no trailing punctuation.
pub fn normalize_sentence(sentence: &str) -> String {
    let mut text = plain_text(sentence).trim().to_lowercase();
    while text.ends_with(['.', '!', '?']) {
        text.pop();
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Distinct normalized sentences (20+ chars) shared by at least two
/// systems; the count of cross-system duplicate sentences.
pub fn cross_system_duplicate_count(docs: &[&SystemDoc]) -> usize {
    let mut occurrences: HashMap<String, usize> = HashMap::new();
    for doc in docs {
        let mut seen = std::collections::BTreeSet::new();
        for sentence in &doc.sentences {
            let normalized = normalize_sentence(sentence);
            if normalized.len() >= 20 {
                seen.insert(normalized);
            }
        }
        for normalized in seen {
            *occurrences.entry(normalized).or_default() += 1;
        }
    }
    occurrences
        .into_values()
        .filter(|systems| *systems >= 2)
        .count()
}

/// JSON value helper: null when `None`.
pub fn opt(value: Option<f64>, digits: u32) -> Value {
    match value {
        Some(v) => json!(round(v, digits)),
        None => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_splits_numbers_words_and_apostrophes() {
        let tokens = tokenize("**51 Peg b** is **about 4.23 days** away.");
        let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            ["51", "Peg", "b", "is", "about", "4.23", "days", "away"]
        );
        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[5].kind, TokenKind::Number);
        assert_eq!(tokens[1].kind, TokenKind::Word);
    }

    #[test]
    fn tokenize_keeps_possessives_whole() {
        let tokens = tokenize("times Earth's mass");
        let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, ["times", "Earth's", "mass"]);
    }

    #[test]
    fn sentences_split_on_ending_punctuation() {
        let sentences =
            split_sentences("One sentence. Another one! A third? Trailing text");
        assert_eq!(sentences.len(), 4);
        assert_eq!(sentences[0], "One sentence.");
        assert_eq!(sentences[3], "Trailing text");
    }

    #[test]
    fn decimals_do_not_split_sentences() {
        let sentences = split_sentences(
            "Its radius is **about 2.5 times Earth's radius**. That is larger.",
        );
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].contains("2.5 times"));
        assert_eq!(sentences[1], "That is larger.");
    }

    #[test]
    fn decimal_periods_at_sentence_end_still_split() {
        let sentences = split_sentences("Orbit ends with 5. Next sentence.");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "Orbit ends with 5.");
    }

    #[test]
    fn soft_wrapped_sentences_keep_whitespace() {
        let sentences = split_sentences("a long sentence\ncontinues here. Next.");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "a long sentence\ncontinues here.");
    }

    #[test]
    fn epic_style_paragraph_sentence_count() {
        let sentences = split_sentences(
            "**EPIC 206042996 b** is a Super-Earth. Its period is \
             **about 5.3 days**, shorter than Earth's roughly 365-day \
             year. Its radius is **about 1.7 Earth radii**.",
        );
        assert_eq!(sentences.len(), 3);
        assert!(sentences[1].contains("5.3 days"));
    }

    #[test]
    fn article_split_separates_title_and_body() {
        let (title, paragraphs, sentences) = split_article(
            "# Host: a title\n\nFirst **bold** para.\n\nSecond para here.\n",
        );
        assert_eq!(title.as_deref(), Some("Host: a title"));
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(sentences.len(), 2);
        assert!(paragraphs[0].contains("**"), "bold markers are preserved");
    }

    #[test]
    fn unbolded_measurements_counts_only_bare_measurement_numbers() {
        let (_, paragraphs, _) = split_article(
            "# T\n\n**about 1.51 days** stays bold. Discovered in 2019 \
             with about 5.3 days unbolded, at 2570 K unbolded, and \
             **about 4060 K** bold again.\n",
        );
        let doc = SystemDoc {
            hostname: "T".into(),
            directory: PathBuf::from("."),
            title: None,
            paragraphs,
            sentences: Vec::new(),
            metadata: None,
            failed: false,
            described: true,
        };
        // "5.3 days" (unit follows) and "2570 K" (unit follows) count;
        // "about 1.51 days", "**4060 K**", the year 2019, the "365-day
        // year" anchor, and the guide range "about 1.5 to" do not.
        assert_eq!(doc.unbolded_measurements(), 2);
    }

    #[test]
    fn unbolded_measurements_flags_about_times_comparisons() {
        let (_, paragraphs, _) = split_article(
            "# T\n\nIt is about 0.227 times as dense as Earth, in a \
             class from about 1.5 to **2 Earth radii**.\n",
        );
        let doc = SystemDoc {
            hostname: "T".into(),
            directory: PathBuf::from("."),
            title: None,
            paragraphs,
            sentences: Vec::new(),
            metadata: None,
            failed: false,
            described: true,
        };
        let phrases = doc.unbolded_measurement_phrases();
        assert_eq!(phrases, vec!["about 0.227".to_owned()]);
    }

    #[test]
    fn unbolded_measurements_ignores_year_anchor() {
        let (_, paragraphs, _) = split_article(
            "# T\n\nIts year is shorter than Earth's roughly 365-day \
             year, at **about 12.3 days**.\n",
        );
        let doc = SystemDoc {
            hostname: "T".into(),
            directory: PathBuf::from("."),
            title: None,
            paragraphs,
            sentences: Vec::new(),
            metadata: None,
            failed: false,
            described: true,
        };
        assert_eq!(doc.unbolded_measurements(), 0);
    }

    #[test]
    fn corpus_load_reads_labeled_variant_files() {
        let root = std::env::temp_dir().join(format!(
            "exodata-analyze-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir_all(root.join("sys")).unwrap();
        fs::write(
            root.join("sys/request.toml"),
            "[system]\nhostname = 'Sys'\n",
        )
        .unwrap();
        fs::write(
            root.join("sys/description.md"),
            "# Served\n\nServed prose here.\n",
        )
        .unwrap();
        fs::write(
            root.join("sys/description_v2.md"),
            "# Variant\n\nVariant prose here.\n",
        )
        .unwrap();
        fs::write(
            root.join("sys/metadata_v2.toml"),
            "hostname = 'Sys'\nattempts = 2\n",
        )
        .unwrap();
        let served = Corpus::load(&root, None).unwrap();
        let variant = Corpus::load(&root, Some("v2")).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(served.systems.len(), 1);
        assert_eq!(variant.systems.len(), 1);
        assert_eq!(served.systems[0].title.as_deref(), Some("Served"));
        assert_eq!(variant.systems[0].title.as_deref(), Some("Variant"));
        assert!(
            variant.systems[0]
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata["attempts"] == 2)
        );
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        let sorted = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&sorted, 50), 3.0);
        assert_eq!(percentile(&sorted, 0), 1.0);
        assert_eq!(percentile(&sorted, 100), 5.0);
    }

    #[test]
    fn counter_merges_frequency_and_documents() {
        let mut counter = Counter::new();
        let mut local = HashMap::new();
        local.insert("a".to_owned(), 3);
        counter.merge(&local);
        let mut local = HashMap::new();
        local.insert("a".to_owned(), 1);
        counter.merge(&local);
        assert_eq!(counter.top(10)[0], ("a".to_owned(), 4, 2));
    }

    #[test]
    fn normalized_sentences_ignore_case_and_padding() {
        assert_eq!(normalize_sentence("  The Star is hot. "), "the star is hot");
    }
}
