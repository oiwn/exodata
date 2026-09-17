//! Deterministic Markdown gate for generated stellar-host descriptions.
use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use serde_json::Value;
use std::{collections::BTreeSet, sync::LazyLock};

/// Request-derived facts the article's numbers and labels are checked
/// against.
pub(super) struct ValidationContext {
    pub planet_names: Vec<String>,
    pub spectral_label: Option<String>,
    pub licensed_tokens: BTreeSet<String>,
    /// Lowercase phrases (for example "circumbinary", "pulsar timing")
    /// licensed by prepared facts; they are removed before the banned
    /// pattern list runs.
    pub licensed_phrases: BTreeSet<String>,
    pub claims: ClaimContext,
}

/// Request-derived scope for host-star measurement checks.
#[derive(Default)]
pub(super) struct ClaimContext {
    stellar_measurements: BTreeSet<&'static str>,
}

impl ClaimContext {
    pub(super) fn from_request(request: &Value) -> Self {
        let mut context = Self::default();
        if request["system"]["catalog_system_star_count"]
            .as_u64()
            .is_some_and(|n| n > 1)
        {
            for quantity in ["age", "mass", "radius", "temperature"] {
                if request["star"]["measurements"][quantity].is_object() {
                    context.stellar_measurements.insert(quantity);
                }
            }
        }
        context
    }
}

static SENTENCE_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[.!?](?:\s+|$)|\n").unwrap());
static EARTH_YEAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bearth(?:'s)?(?:\s+(?:roughly|about))?(?:\s+365(?:-|\s+)day)?[- ]+years?\b|\b(?:shorter|longer)\s+than\s+earth's\b").unwrap()
});

pub(super) fn is_earth_year_reference(text: &str) -> bool {
    EARTH_YEAR.is_match(&text.replace("**", "").replace('’', "'").to_lowercase())
}
static SYSTEM_MEASUREMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:(?:system(?:'s)?|system-wide)\s+(?:(?:also|has|a|an|the|reported|estimated|mean|total|combined|stellar)\s+){0,6}(?P<subject>age|mass|radius|temperature)|(?P<object>age|mass|radius|temperature)\s+(?:of|for)\s+(?:the\s+)?(?:whole\s+|entire\s+)?system)\b").unwrap()
});

static STELLAR_MINIMUM_MASS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bminimum[- ]mass(?:\s+(?:quantity|of|is|about|at|a|reported|upper|lower|limit|has)){0,8}\s+\d+(?:\.\d+)?\s+solar masses\b").unwrap()
});

fn claim_violations(markdown: &str, ctx: &ValidationContext) -> Vec<String> {
    let text = markdown.replace("**", "").replace('’', "'").to_lowercase();
    let mut errors = Vec::new();
    for sentence in SENTENCE_END
        .split(&text)
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if is_earth_year_reference(sentence) {
            errors.push(format!("Earth-year comparisons and conversions are omitted: '{sentence}'; remove the Earth reference and retain the supplied orbital period in days"));
        }
        if STELLAR_MINIMUM_MASS.is_match(sentence) {
            errors.push(format!("stellar mass is not a minimum mass: '{sentence}'; retain the host star's supplied mass estimate or bound"));
        }
        for matched in SYSTEM_MEASUREMENT.captures_iter(sentence) {
            let quantity = matched
                .name("subject")
                .or_else(|| matched.name("object"))
                .unwrap()
                .as_str();
            if ctx.claims.stellar_measurements.contains(quantity) {
                errors.push(format!("host-star {quantity} attributed to the whole multi-star system: '{sentence}'; attribute the measurement to the selected host star"));
            }
        }
    }
    errors
}

/// Full deterministic gate chain: Markdown structure, bolded names and
/// measurements, punctuation, numeric allowlist, and banned phrasings.
pub(super) fn validate_article(
    markdown: &str,
    ctx: &ValidationContext,
) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    violations.extend(claim_violations(markdown, ctx));
    let plain = markdown.replace("**", "");
    for name in &ctx.planet_names {
        if !plain.contains(name) {
            violations.push(format!("missing requested planet: {name}"));
        }
    }
    if let Err(errors) = validate(markdown, &ctx.planet_names) {
        violations.extend(errors);
    }
    if let Err(errors) =
        unbolded_measurements(markdown, ctx.spectral_label.as_deref())
    {
        violations.extend(errors);
    }
    if let Err(errors) = banned_patterns(markdown, &ctx.licensed_phrases) {
        violations.extend(errors);
    }
    if let Err(errors) = numeric_allowlist(markdown, &ctx.licensed_tokens) {
        violations.extend(errors);
    }
    if violations.is_empty() {
        Ok(())
    } else {
        let mut unique = violations;
        unique.dedup();
        Err(unique)
    }
}

/// Measurement phrases (a number with its unit, or the spectral label)
/// must appear inside bold spans.
pub(super) fn unbolded_measurements(
    markdown: &str,
    spectral_label: Option<&str>,
) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let parser = Parser::new_ext(markdown, Options::empty());
    let mut in_title = false;
    let mut strong_depth = 0usize;
    let mut strong_id = 0usize;
    let mut paragraph = String::new();
    let mut bold = Vec::new();
    for event in parser {
        match event {
            Event::Text(text) if !in_title => {
                paragraph.push_str(&text);
                bold.extend(std::iter::repeat_n(
                    if strong_depth > 0 { strong_id } else { 0 },
                    text.len(),
                ));
            }
            Event::SoftBreak | Event::HardBreak if !in_title => {
                paragraph.push(' ');
                bold.push(if strong_depth > 0 { strong_id } else { 0 });
            }
            Event::Start(Tag::Heading { .. }) => in_title = true,
            Event::Start(Tag::Strong) => {
                if strong_depth == 0 {
                    strong_id += 1;
                }
                strong_depth += 1;
            }
            Event::End(TagEnd::Strong) => {
                strong_depth = strong_depth.saturating_sub(1)
            }
            Event::End(TagEnd::Heading(_)) if in_title => in_title = false,
            Event::End(TagEnd::Paragraph) => {
                for range in measurement_ranges(&paragraph) {
                    if bold[range.start] == 0
                        || !bold[range.clone()]
                            .iter()
                            .all(|id| *id == bold[range.start])
                    {
                        violations.push(format!("measurement phrase '{}' must be bolded as one complete phrase", &paragraph[range]));
                    }
                }
                if let Some(label) =
                    spectral_label.filter(|label| !label.is_empty())
                {
                    for (start, _) in paragraph.match_indices(label) {
                        if bold[start] == 0
                            || !bold[start..start + label.len()]
                                .iter()
                                .all(|id| *id == bold[start])
                        {
                            violations.push(format!(
                                "spectral label '{label}' must be bolded"
                            ));
                        }
                    }
                }
                paragraph.clear();
                bold.clear();
            }
            _ => {}
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

const UNIT_PREFIXES: &[&str] = &[
    "earth masses",
    "earth mass",
    "earth radii",
    "earth radius",
    "solar masses",
    "solar mass",
    "solar radii",
    "solar radius",
    "light-years",
    "light-year",
    "billion years",
    "billion year",
    "million years",
    "million year",
    "g/cm³",
    "g/cm",
    "times jupiter's",
    "times jupiter",
    "times earth's",
    "times earth",
    "days",
    "day",
];

/// Find the first "number unit" phrase in a plain-text run, for example
/// "5.6 Earth masses" or "2570 K". A trailing "K" unit only counts when
/// not followed by a letter, so "2 known" does not match.
fn number_with_unit(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        let mut dotted = false;
        while i < chars.len()
            && (chars[i].is_ascii_digit()
                || (chars[i] == '.'
                    && !dotted
                    && i + 1 < chars.len()
                    && chars[i + 1].is_ascii_digit()))
        {
            if chars[i] == '.' {
                dotted = true;
            }
            i += 1;
        }
        let number: String = chars[start..i].iter().collect();
        let mut j = i;
        while j < chars.len() && chars[j] == ' ' {
            j += 1;
        }
        let rest: String = chars[j..].iter().collect::<String>().to_lowercase();
        let matched = UNIT_PREFIXES.iter().any(|unit| rest.starts_with(unit))
            || (rest.starts_with('k')
                && !rest[1..].chars().next().is_some_and(char::is_alphabetic));
        if matched {
            let mut unit_len = 1;
            for unit in UNIT_PREFIXES {
                if rest.starts_with(unit) {
                    unit_len = unit.chars().count();
                    break;
                }
            }
            let mut phrase = number;
            for c in &chars[i..j] {
                phrase.push(*c);
            }
            phrase.push_str(
                &text.chars().skip(j).take(unit_len).collect::<String>(),
            );
            if start >= 6
                && chars[start - 6..start]
                    .iter()
                    .collect::<String>()
                    .eq_ignore_ascii_case("about ")
            {
                phrase = format!(
                    "{}{phrase}",
                    chars[start - 6..start].iter().collect::<String>()
                );
            }
            return Some(phrase.trim().to_owned());
        }
    }
    None
}

fn measurement_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut offset = 0;
    while let Some(phrase) = number_with_unit(&text[offset..]) {
        let start = offset
            + text[offset..]
                .find(&phrase)
                .expect("measurement is a substring");
        offset = start + phrase.len();
        ranges.push(start..offset);
    }
    ranges
}

/// Preserve existing strong spans while extending them over complete
/// measurements. Work on visible text so a misplaced closing ** cannot
/// hide the rest of a unit from the matcher.
fn repair_measurement_emphasis(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let mut plain = String::new();
        let mut bold = Vec::new();
        for (index, segment) in line.split("**").enumerate() {
            plain.push_str(segment);
            bold.extend(std::iter::repeat_n(index % 2 == 1, segment.len()));
        }
        // Leave malformed strong markup to the Markdown gate.
        if line.matches("**").count() % 2 != 0 {
            result.push_str(line);
            continue;
        }
        for range in measurement_ranges(&plain) {
            bold[range].fill(true);
        }
        let mut inside = false;
        for (index, character) in plain.char_indices() {
            if bold[index] != inside {
                result.push_str("**");
                inside = bold[index];
            }
            result.push(character);
        }
        if inside {
            result.push_str("**");
        }
    }
    result
}

/// Substrings and standalone words that deterministically indicate a
/// banned inference or commentary family, with the rule reported back.
const BANNED_PATTERNS: &[(&str, &str)] = &[
    ("moving outward", "orbital-position wording"),
    ("moving inward", "orbital-position wording"),
    ("farther out", "orbital-position wording"),
    ("closer in", "orbital-position wording"),
    ("innermost", "orbital-position wording"),
    ("outermost", "orbital-position wording"),
    ("whips", "orbital-speed wording"),
    ("races around", "orbital-speed wording"),
    ("races through", "orbital-speed wording"),
    ("follows", "ordering wording"),
    ("notable", "significance or hype wording"),
    ("pioneering", "significance or hype wording"),
    ("remarkable", "significance or hype wording"),
    ("renowned", "significance or hype wording"),
    ("famous", "significance or hype wording"),
    ("iconic", "significance or hype wording"),
    ("tight orbit", "orbit characterization"),
    ("hot jupiter", "unlicensed classification"),
    ("circumbinary", "unlicensed orbit classification"),
    ("sun-like", "unlicensed host classification"),
    ("pulsar", "unlicensed host-type label"),
    ("young", "interpretive age label"),
    ("modest", "evaluative wording"),
    ("about about", "doubled hedge wording"),
    ("the guide", "meta commentary"),
    ("the request", "meta commentary"),
    ("the evidence", "meta commentary"),
    ("supplied", "preparation language"),
    ("recognized second", "ordering wording"),
    ("confirmed", "unsourced status claim"),
    ("not reported", "missing-field commentary"),
    ("unreported", "missing-field commentary"),
    ("unlisted", "missing-field commentary"),
    ("unavailable", "missing-field commentary"),
    ("not supplied", "missing-field commentary"),
    ("not been measured", "missing-field commentary"),
    ("not been detected", "missing-field commentary"),
    ("has been measured", "missing-field commentary"),
    ("remains unknown", "missing-field commentary"),
    ("remains uncharacterized", "missing-field commentary"),
    ("was not specified", "missing-field commentary"),
    ("uncertainty spanning", "raw-uncertainty derivation"),
    ("uncertainties spanning", "raw-uncertainty derivation"),
    ("consistent with", "unsupported interpretation"),
    ("not constrained", "missing-field commentary"),
];

const BANNED_WORDS: &[&str] = &["inner", "outer"];

// The supplied "very young" class also licenses the weaker age label, but
// must not license comparative/superlative words such as younger/youngest.
static YOUNG_AGE_LABEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:very\s+)?young\b").unwrap());

pub(super) fn banned_patterns(
    markdown: &str,
    licensed: &BTreeSet<String>,
) -> Result<(), Vec<String>> {
    let mut text = markdown.replace("**", "").to_lowercase();
    for phrase in licensed {
        let phrase = phrase.to_lowercase();
        if phrase == "very young" {
            text = YOUNG_AGE_LABEL.replace_all(&text, " ").into_owned();
        } else if !phrase.is_empty() {
            text = text.replace(&phrase, " ");
        }
    }
    let mut violations = Vec::new();
    for (pattern, rule) in BANNED_PATTERNS {
        if text.contains(pattern) {
            violations.push(format!("prohibited phrasing '{pattern}' ({rule})"));
        }
    }
    let normalized: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let padded = format!(" {normalized} ");
    for word in BANNED_WORDS {
        if padded.contains(&format!(" {word} ")) {
            violations.push(format!(
                "prohibited word '{word}' (orbital-position wording)"
            ));
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Every numeric token in the article must be licensed by request-derived
/// facts: display strings, comparisons, guide constants, years, counts, or
/// object names.
pub(super) fn numeric_allowlist(
    markdown: &str,
    licensed: &BTreeSet<String>,
) -> Result<(), Vec<String>> {
    let present = numeric_tokens(markdown);
    let extra: Vec<&String> = present.difference(licensed).collect();
    if extra.is_empty() {
        Ok(())
    } else {
        Err(vec![format!(
            "numbers not licensed by the supplied facts: {}",
            extra
                .iter()
                .map(|token| token.as_str())
                .take(8)
                .collect::<Vec<_>>()
                .join(", ")
        )])
    }
}

/// Validate a generated article. Allowed shape: one level-one title at the
/// start followed by paragraphs, with only plain text, bold, and emphasis.
/// Every planet-name occurrence must be bolded, and em dashes or double
/// hyphens are rejected. Returns all violations so they can be fed back.
pub(super) fn validate(
    markdown: &str,
    planet_names: &[String],
) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    if markdown.contains('—') || markdown.contains("--") {
        violations.push(
            "em dashes and double hyphens are not allowed; use a single \
             hyphen or rewrite the sentence"
                .into(),
        );
    }
    let mut parser = Parser::new_ext(markdown, Options::empty()).peekable();
    match parser.next() {
        Some(Event::Start(Tag::Heading {
            level: HeadingLevel::H1,
            ..
        })) => {}
        _ => violations.push(
            "the article must start with a single level-one title heading".into(),
        ),
    }
    let mut title_text = String::new();
    let mut in_title = true;
    let mut strong_depth = 0usize;
    for event in parser {
        match event {
            Event::Text(text) => {
                if in_title {
                    title_text.push_str(&text);
                } else if strong_depth == 0 {
                    for name in planet_names {
                        if text.contains(name.as_str()) {
                            violations.push(format!(
                                "planet name {name} must be bolded on every \
                                 occurrence"
                            ));
                        }
                    }
                }
            }
            Event::Code(_) => {
                if !in_title {
                    violations.push("code spans are not allowed".into());
                }
            }
            Event::Start(Tag::Strong) => strong_depth += 1,
            Event::End(TagEnd::Strong) => {
                strong_depth = strong_depth.saturating_sub(1)
            }
            Event::End(TagEnd::Heading(_)) if in_title => in_title = false,
            Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph) => {}
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {}
            Event::SoftBreak | Event::HardBreak => {}
            other => violations.push(format!(
                "only paragraphs, bold, and emphasis are allowed after the \
                 title (found {})",
                describe(other)
            )),
        }
    }
    if title_text.trim().is_empty() {
        violations.push("the title heading must not be empty".into());
    }
    if violations.is_empty() {
        Ok(())
    } else {
        let mut unique = violations;
        unique.dedup();
        Err(unique)
    }
}

/// Collect numeric tokens: runs of ASCII digits with at most one interior
/// dot, after removing bold markers and digit-grouping commas.
pub(super) fn numeric_tokens(markdown: &str) -> BTreeSet<String> {
    let cleaned = markdown.replace("**", "").replace(',', "");
    let mut tokens = BTreeSet::new();
    let mut current = String::new();
    let mut dotted = false;
    let mut flush = |current: &mut String, dotted: &mut bool| {
        if current.ends_with('.') {
            current.pop();
        }
        if !current.is_empty() {
            tokens.insert(current.clone());
        }
        current.clear();
        *dotted = false;
    };
    for c in cleaned.chars() {
        if c.is_ascii_digit() {
            current.push(c);
        } else if c == '.' && !dotted && !current.is_empty() {
            current.push(c);
            dotted = true;
        } else {
            flush(&mut current, &mut dotted);
        }
    }
    flush(&mut current, &mut dotted);
    tokens
}

static MEASUREMENT_ABOUT: LazyLock<Regex> = LazyLock::new(|| {
    let number = r"(?:\*\*)?(?:[0-9]+(?:[.,](?:\*\*)?[0-9]+)*|one\b|two\b|three\b|four\b|five\b|six\b|seven\b|eight\b|nine\b|ten\b|half\b|twice\b)(?:\*\*)?";
    let unit = r"(?:earth\b|solar\b|light-years?\b|billion\b|million\b|years?\b|days?\b|hours?\b|minutes?\b|kelvin\b|k\b|g/cm|grams?\b|times\b|(?:the[\t ]+)?sun(?:'s|’s)?\b)";
    Regex::new(&format!(r"(?i)\babout[\t ]+(?P<quantity>(?:every[\t ]+)?{number}(?:[\t ]*(?:,|and\b|to\b|-)[\t ]*(?:and[\t ]+)?{number})*[\t ]+{unit})")).unwrap()
});

/// Remove the approximation prefix from quantities, preserving ordinary
/// prepositions ("about its composition") and every value/unit/bound.
pub(super) fn remove_measurement_about(text: &str) -> String {
    MEASUREMENT_ABOUT
        .replace_all(text, "${quantity}")
        .into_owned()
}

/// Idempotent presentation pass: punctuation, redundant hedges, complete
/// measurement/name bolding, possessives, and plain measurement values.
pub(super) fn normalize_article(
    markdown: &str,
    spectral_label: Option<&str>,
    planet_names: &[String],
) -> String {
    let markdown =
        fix_dashes(&fix_doubled_hedge(&fix_qualifier_hedges(markdown)));
    let (title, body) = match markdown.split_once("\n\n") {
        Some((title, body)) => (title, body),
        None => {
            return remove_measurement_about(&bold_plain_phrases(
                &fix_earth_possessive(&markdown),
                spectral_label,
                planet_names,
            ));
        }
    };
    let body = bold_plain_phrases(
        &fix_earth_possessive(body),
        spectral_label,
        planet_names,
    );
    remove_measurement_about(&format!("{title}\n\n{body}"))
}

/// Replace em dashes and double hyphens: digit-adjacent dashes become a
/// plain hyphen (numeric ranges), everything else becomes a spaced
/// single hyphen, matching the house style. Idempotent.
fn fix_dashes(markdown: &str) -> String {
    let chars: Vec<char> = markdown.chars().collect();
    let mut out = String::with_capacity(markdown.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let is_double = c == '-' && chars.get(i + 1) == Some(&'-');
        if c == '—' || is_double {
            let mut end = if is_double { i + 2 } else { i + 1 };
            let previous_digit =
                out.chars().last().is_some_and(|p| p.is_ascii_digit());
            let next_digit = chars.get(end).is_some_and(|n| n.is_ascii_digit());
            if previous_digit && next_digit {
                out.push('-');
            } else {
                while out.ends_with(' ') {
                    out.pop();
                }
                out.push_str(" - ");
                while chars.get(end) == Some(&' ') {
                    end += 1;
                }
            }
            i = end;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Collapse any run of repeated "about" (for example "about about")
/// to a single occurrence, preserving the first word's case.
/// Measurement-context words after which a preceding "estimated" is a
/// redundant hedge (the qualifier's meaning lives in the display
/// strings, bounds, and provenance wording, which stay untouched).
const HEDGE_CONTEXT_WORDS: &[&str] = &[
    "mean density",
    "minimum-mass quantity",
    "radius",
    "radii",
    "mass",
    "masses",
    "density",
    "age",
    "temperature",
    "period",
    "distance",
    "year",
    "years",
];

/// Collapse the redundant "estimated" qualifier before measurement
/// contexts: "Its estimated radius of X" → "Its radius of X", "an
/// estimated radius" → "a radius" (article agreement kept), "an
/// estimated about X" → "about X". Other uses ("estimated to be
/// rare") are left alone. Idempotent and fact-preserving.
fn fix_qualifier_hedges(markdown: &str) -> String {
    let chars: Vec<char> = markdown.chars().collect();
    let mut out = String::with_capacity(markdown.len());
    let mut i = 0;
    let is_word_char = |c: char| c.is_ascii_alphanumeric() || c == '\'';
    while i < chars.len() {
        let window: String = chars[i..(i + 9).min(chars.len())].iter().collect();
        if window.eq_ignore_ascii_case("estimated")
            && chars.len() >= i + 9
            && (i == 0 || !is_word_char(chars[i - 1]))
        {
            let after = i + 9;
            let leading_space = chars.get(after) == Some(&' ');
            let rest_start = if leading_space { after + 1 } else { after };
            let rest: String = chars[rest_start..]
                .iter()
                .collect::<String>()
                .to_lowercase();
            let context_word = HEDGE_CONTEXT_WORDS.iter().find(|word| {
                rest.starts_with(*word)
                    && !rest[word.len()..]
                        .chars()
                        .next()
                        .is_some_and(is_word_char)
            });
            let about_follows = rest.starts_with("about")
                && !rest[5..].chars().next().is_some_and(is_word_char);
            let bold_follows = chars.get(rest_start) == Some(&'*');
            // "Mass is estimated at about X" / "estimated at **X**".
            let at_measurement =
                rest.starts_with("at about ") || rest.starts_with("at *");
            if context_word.is_some()
                || about_follows
                || bold_follows
                || at_measurement
            {
                // Preceding article: drop or repair agreement.
                let trimmed = out.trim_end_matches(' ');
                let ends_with_an = trimmed.len() >= 2
                    && trimmed
                        .chars()
                        .skip(trimmed.len() - 2)
                        .collect::<String>()
                        .eq_ignore_ascii_case("an")
                    && (trimmed.len() == 2
                        || !is_word_char(
                            trimmed
                                .chars()
                                .nth(trimmed.len() - 3)
                                .expect("len > 2 checked"),
                        ));
                if ends_with_an {
                    out = trimmed[..trimmed.len() - 2].to_owned();
                    if let Some(word) = context_word {
                        let article =
                            if word.starts_with(['a', 'e', 'i', 'o', 'u']) {
                                "an "
                            } else {
                                "a "
                            };
                        out.push_str(article);
                    } else {
                        // "an estimated about X" / "an estimated **X**":
                        // drop the article with the hedge.
                    }
                }
                // Drop "estimated" (and "at" for the verb form), plus
                // the trailing spaces.
                i = after;
                if leading_space {
                    let mut skip = 1;
                    if at_measurement {
                        skip += 3;
                    }
                    i = after + skip;
                    if !out.ends_with(' ')
                        && !out.is_empty()
                        && chars.get(i).is_some_and(|c| !c.is_whitespace())
                    {
                        out.push(' ');
                    }
                }
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn fix_doubled_hedge(markdown: &str) -> String {
    let chars: Vec<char> = markdown.chars().collect();
    let matches_about = |from: usize| {
        chars.get(from..from + 5).is_some_and(|window| {
            window
                .iter()
                .collect::<String>()
                .eq_ignore_ascii_case("about")
        })
    };
    let mut out = String::with_capacity(markdown.len());
    let mut i = 0;
    while i < chars.len() {
        if matches_about(i)
            && chars.get(i + 5) == Some(&' ')
            && matches_about(i + 6)
        {
            out.extend(&chars[i..i + 5]);
            let mut end = i + 11;
            while chars.get(end) == Some(&' ') && matches_about(end + 1) {
                end += 6;
            }
            i = end;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn bold_in_segment(
    segment: &str,
    spectral_label: Option<&str>,
    planet_names: &[String],
) -> String {
    let mut wrapped = String::with_capacity(segment.len());
    let mut rest = segment;
    while let Some(phrase) = number_with_unit(rest) {
        let position = rest.find(&phrase).unwrap_or(0);
        wrapped.push_str(&rest[..position]);
        wrapped.push_str("**");
        wrapped.push_str(&phrase);
        wrapped.push_str("**");
        rest = &rest[position + phrase.len()..];
    }
    wrapped.push_str(rest);
    let wrapped = match spectral_label {
        Some(label) if !label.is_empty() => {
            wrapped.replace(label, &format!("**{label}**"))
        }
        _ => wrapped,
    };
    bold_names_in_plain_text(&wrapped, planet_names)
}

/// Wrap every planet-name occurrence (longest first) in bold, skipping
/// spans that are already strong.
fn bold_names_in_plain_text(text: &str, planet_names: &[String]) -> String {
    if planet_names.is_empty() {
        return text.to_owned();
    }
    let mut names: Vec<&String> = planet_names.iter().collect();
    names.sort_by_key(|name| std::cmp::Reverse(name.len()));
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find("**") {
        out.push_str(&bold_names_in_run(&rest[..index], &names));
        out.push_str("**");
        rest = &rest[index + 2..];
        match rest.find("**") {
            Some(close) => {
                out.push_str(&rest[..close + 2]);
                rest = &rest[close + 2..];
            }
            None => {
                out.push_str(rest);
                rest = "";
                break;
            }
        }
    }
    out.push_str(&bold_names_in_run(rest, &names));
    out
}

fn bold_names_in_run(run: &str, names: &[&String]) -> String {
    let mut out = run.to_owned();
    for name in names {
        let plain = name.replace("**", "");
        if plain.is_empty() {
            continue;
        }
        out = out.replace(&plain, &format!("**{plain}**"));
    }
    out
}

fn bold_plain_phrases(
    text: &str,
    spectral_label: Option<&str>,
    planet_names: &[String],
) -> String {
    let text = repair_measurement_emphasis(text);
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let mut rest = line;
        let mut inside_strong = false;
        while let Some(index) = rest.find("**") {
            let (segment, tail) = rest.split_at(index);
            if inside_strong {
                out.push_str(segment);
            } else {
                out.push_str(&bold_in_segment(
                    segment,
                    spectral_label,
                    planet_names,
                ));
            }
            out.push_str("**");
            rest = &tail[2..];
            inside_strong = !inside_strong;
        }
        if inside_strong {
            out.push_str(rest);
        } else {
            out.push_str(&bold_in_segment(rest, spectral_label, planet_names));
        }
    }
    out
}

fn fix_earth_possessive(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find("times Earth") {
        out.push_str(&rest[..position]);
        out.push_str("times Earth's");
        rest = &rest[position + "times Earth".len()..];
        // The replacement is idempotent: skip a possessive suffix that
        // is already present, ASCII or typographic.
        if let Some(stripped) = rest
            .strip_prefix("'s")
            .or_else(|| rest.strip_prefix("\u{2019}s"))
        {
            rest = stripped;
        }
    }
    out.push_str(rest);
    out
}

fn describe(event: Event<'_>) -> String {
    match event {
        Event::Start(tag) => format!("{tag:?}"),
        Event::End(tag) => format!("{tag:?}"),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["TRAPPIST-1 b".to_owned(), "TRAPPIST-1 c".to_owned()]
    }

    #[test]
    fn accepts_valid_markdown() {
        let markdown = "# TRAPPIST-1 and its planets\n\n**TRAPPIST-1 b** is \
short with **about 1.51 days** - a hyphen is fine. **TRAPPIST-1 c** \
follows.";
        assert!(validate(markdown, &names()).is_ok());
    }

    #[test]
    fn rejects_missing_or_extra_headings_and_non_paragraph_blocks() {
        assert!(validate("No title paragraph.", &names()).is_err());
        assert!(validate("# Title\n\n## Section\n\nBody", &names()).is_err());
        assert!(validate("# Title\n\n- item", &names()).is_err());
        assert!(validate("# Title\n\n[link](https://x)", &names()).is_err());
        assert!(validate("# Title\n\n`code`", &names()).is_err());
    }

    #[test]
    fn rejects_em_dashes_and_double_hyphens() {
        assert!(validate("# T\n\nOne - two - and three.", &names()).is_ok());
        assert!(
            validate("# T\n\norbit - its year - in far less.", &names()).is_ok()
        );
        assert!(validate("# T\n\norbit -- its year.", &names()).is_err());
        assert!(validate("# T\n\norbit — its year.", &names()).is_err());
        let errors = validate("# T\n\nan em dash: x — y.", &names()).unwrap_err();
        assert!(errors.iter().any(|v| v.contains("em dashes")));
    }

    #[test]
    fn rejects_unbolded_planet_names() {
        let errors = validate(
            "# TRAPPIST-1 and its planets\n\nTRAPPIST-1 b is innermost.",
            &names(),
        )
        .unwrap_err();
        assert!(
            errors
                .iter()
                .any(|v| v.contains("TRAPPIST-1 b must be bolded"))
        );
        assert!(validate("# T\n\n**TRAPPIST-1 b** orbits.", &names()).is_ok());
    }

    #[test]
    fn measurement_bolding_gate_flags_plain_values_only() {
        assert!(
            unbolded_measurements(
                "# T\n\n**TRAPPIST-1 b** orbits in **about 1.51 days**, at \
             **2570 K** and **5.45 g/cm³**.",
                Some("M8.0 V"),
            )
            .is_ok()
        );
        let errors = unbolded_measurements(
            "# T\n\nIt orbits in about 1.51 days at spectral type M8.0 V.",
            Some("M8.0 V"),
        )
        .unwrap_err();
        assert!(errors.iter().any(|v| v.contains("1.51 days")));
        assert!(errors.iter().any(|v| v.contains("M8.0 V")));
        assert!(
            unbolded_measurements(
                "# T\n\nTwo known planets and 2 known companions.",
                None
            )
            .is_ok()
        );
        assert!(
            unbolded_measurements(
                "# T\n\n**0.63 solar masses** and about 0.769 solar radii.",
                None
            )
            .is_err()
        );
    }

    #[test]
    fn banned_patterns_reject_known_regressions() {
        let unlicensed = BTreeSet::new();
        for text in [
            "Moving outward, X",
            "Farther out, X",
            "the outermost planet",
            "an inner world",
            "whips around its star",
            "one of the most notable exoplanets",
            "a pioneering hot jupiter",
            "a massive pulsar",
            "a young star",
            "a circumbinary planet",
            "a Sun-like star",
            "about **about 4 days**",
            "the threshold that the guide uses",
            "with a supplied spectral type",
            "the request states",
            "according to the evidence",
            "no orbital period has been measured",
            "uncertainty spanning about 1.11 to 1.88",
            "planets have been confirmed",
        ] {
            assert!(
                banned_patterns(&format!("# T\n\n{text}"), &unlicensed).is_err(),
                "{text}"
            );
        }
        assert!(
            banned_patterns(
                "# T\n\nA star with a short year; a dense planet; the far edge \
             of the dataset is unknown here.",
                &unlicensed
            )
            .is_ok()
        );
    }

    #[test]
    fn licensed_phrases_lift_only_their_own_bans() {
        let unlicensed = BTreeSet::new();
        let mut licensed = BTreeSet::from(["circumbinary".to_owned()]);
        assert!(
            banned_patterns("# T\n\na circumbinary planet", &licensed).is_ok()
        );
        assert!(
            banned_patterns("# T\n\na circumbinary planet", &unlicensed).is_err()
        );
        licensed.insert("sun-like".to_owned());
        assert!(banned_patterns("# T\n\na Sun-like star", &licensed).is_ok());
        assert!(banned_patterns("# T\n\na Sun-like star", &unlicensed).is_err());
        let licensed = BTreeSet::from(["very young".to_owned()]);
        assert!(banned_patterns("# T\n\na very young star", &licensed).is_ok());
        assert!(banned_patterns("# T\n\na young star", &licensed).is_ok());
        assert!(
            banned_patterns("# A Young Red Dwarf\n\nA star.", &licensed).is_ok()
        );
        assert!(
            banned_patterns("# A Young Red Dwarf\n\nA star.", &unlicensed)
                .is_err()
        );
        for word in ["younger", "youngest", "very younger", "very youngest"] {
            assert!(
                banned_patterns(&format!("# T\n\nA {word} star."), &licensed)
                    .is_err(),
                "{word}"
            );
        }
        let licensed = BTreeSet::from(["pulsar timing".to_owned()]);
        assert!(
            banned_patterns("# T\n\ndetected by pulsar timing", &licensed)
                .is_ok()
        );
        assert!(banned_patterns("# T\n\na massive pulsar", &licensed).is_err());
        let licensed = BTreeSet::from(["pulsar".to_owned()]);
        assert!(banned_patterns("# T\n\na massive pulsar", &licensed).is_ok());
    }

    #[test]
    fn qualifier_hedges_collapse_before_measurements() {
        let cases = [
            ("Its estimated radius of X", "Its radius of X"),
            ("with an estimated radius", "with a radius"),
            ("an estimated age of X", "an age of X"),
            ("At an estimated about 2570 K", "At about 2570 K"),
            ("The estimated mean density is X", "The mean density is X"),
            (
                "an estimated **about 1.13 Earth radii**",
                "**about 1.13 Earth radii**",
            ),
            ("an estimated year of X", "a year of X"),
            (
                "an estimated minimum-mass quantity of X",
                "a minimum-mass quantity of X",
            ),
            ("Their estimated years are X", "Their years are X"),
            (
                "Mass is estimated at about 0.96 Earth masses",
                "Mass is about 0.96 Earth masses",
            ),
            (
                "Mass is estimated at **about 5.2 Earth masses**",
                "Mass is **about 5.2 Earth masses**",
            ),
            (
                "the value is estimated to be rare",
                "the value is estimated to be rare",
            ),
            ("an underestimated radius", "an underestimated radius"),
        ];
        for (input, expected) in cases {
            assert_eq!(fix_qualifier_hedges(input), expected, "{input}");
        }
        let once = fix_qualifier_hedges("Its estimated radius of X");
        assert_eq!(fix_qualifier_hedges(&once), once, "idempotent");
    }

    #[test]
    fn normalize_bolds_measurements_and_repairs_possessives() {
        let normalized = normalize_article(
            "# Title with 3 planets\n\n**Planet b** orbits in about 1.51 \
             days at 2570 K, type M8.0 V, 0.989 times Earth. Already \
             **about 5 days** stays once.",
            Some("M8.0 V"),
            &["Planet b".to_owned()],
        );
        assert!(normalized.contains("# Title with 3 planets\n\n"));
        assert!(normalized.contains("**1.51 days**"));
        assert!(normalized.contains("**2570 K**"));
        assert!(normalized.contains("**M8.0 V**"));
        assert!(normalized.contains("**0.989 times Earth's**"));
        assert!(normalized.contains("Already **5 days** stays once."));
        assert!(!normalized.contains("times Earth."));
        assert_eq!(
            normalize_article(
                &normalized,
                Some("M8.0 V"),
                &["Planet b".to_owned()],
            ),
            normalized,
            "normalize must be idempotent"
        );
        let untouched = normalize_article("no body split", None, &[]);
        assert_eq!(untouched, "no body split");
    }

    #[test]
    fn normalize_fixes_dashes_and_doubled_hedges() {
        let normalized = normalize_article(
            "# Drafter wobble — a test\n\nA range of 5—10 days, a break — \
             like this — and a double--hyphen. The value is about about \
             3.2 days for **Planet b**.",
            None,
            &["Planet b".to_owned()],
        );
        assert!(normalized.contains("5-**10 days**"));
        assert!(normalized.contains("a break - like this -"));
        assert!(normalized.contains("double - hyphen"));
        assert!(normalized.contains("**3.2 days**"));
        assert!(!normalized.contains("about about"));
        assert!(!normalized.contains('—'));
        assert_eq!(
            normalize_article(&normalized, None, &["Planet b".to_owned()]),
            normalized,
            "dash and hedge fixes are idempotent"
        );
    }

    #[test]
    fn normalize_bolds_planet_names_outside_strong_and_title() {
        let normalized = normalize_article(
            "# Kepler-11 and Its Planets\n\nKepler-11 b orbits, and \
             Kepler-11 c follows, while **Kepler-11 d** stays bold. \
             A spectral K2 V label appears once.",
            Some("K2 V"),
            &["Kepler-11 b".to_owned(), "Kepler-11 c".to_owned()],
        );
        assert!(normalized.starts_with("# Kepler-11 and Its Planets\n"));
        assert!(normalized.contains("**Kepler-11 b** orbits"));
        assert!(normalized.contains("**Kepler-11 c** follows"));
        assert!(normalized.contains("**Kepler-11 d** stays bold"));
        assert!(normalized.contains("**K2 V**"));
        assert_eq!(
            normalize_article(
                &normalized,
                Some("K2 V"),
                &["Kepler-11 b".to_owned(), "Kepler-11 c".to_owned()],
            ),
            normalized,
            "name bolding is idempotent"
        );
    }

    #[test]
    fn possessive_repair_is_idempotent_for_both_apostrophes() {
        assert_eq!(
            fix_earth_possessive("0.989 times Earth's mass"),
            "0.989 times Earth's mass"
        );
        assert_eq!(
            fix_earth_possessive("0.989 times Earth\u{2019}s mass"),
            "0.989 times Earth's mass"
        );
    }

    #[test]
    fn numeric_allowlist_accepts_licensed_tokens_only() {
        let mut licensed = numeric_tokens(
            "**about 1.51 days** and **0.989 times Earth's**, in 2016",
        );
        licensed.insert("365".to_owned());
        assert!(
            numeric_allowlist(
                "# T\n\n**about 1.51 days**, **0.989 times Earth's**, 2016, and \
             365-day phrasing",
                &licensed
            )
            .is_ok()
        );
        let errors = numeric_allowlist("# T\n\nSpans 1.11 to 1.88", &licensed)
            .unwrap_err();
        assert!(errors.iter().any(|v| v.contains("1.11")));
    }

    #[test]
    fn semantic_patterns_from_retired_narrative_gate_still_fail() {
        for text in [
            "Its density is consistent with rock.",
            "Its mass is not constrained.",
        ] {
            assert!(banned_patterns(text, &BTreeSet::new()).is_err());
        }
    }

    #[test]
    fn dense_repeated_measurements_are_not_rejected_for_style() {
        let article = "# Pair\n\n**Planet b** has **about 1 Earth masses**, \
            **about 1 Earth radii**, and **about 1 days**. \
            **Planet c** has **about 1 Earth masses**, **about 1 Earth radii**, \
            and **about 1 days**. Its mass is **about 1 Earth masses**. \
            Its radius is **about 1 Earth radii**. Its period is **about 1 days**.";
        let ctx = ValidationContext {
            planet_names: vec!["Planet b".into(), "Planet c".into()],
            spectral_label: None,
            licensed_tokens: numeric_tokens("1"),
            licensed_phrases: BTreeSet::new(),
            claims: ClaimContext::default(),
        };
        assert!(validate_article(article, &ctx).is_ok());
    }

    fn claim_context() -> ValidationContext {
        let planet_names = vec![
            "L 98-59 b".into(),
            "L 98-59 e".into(),
            "TOI-178 f".into(),
            "TRAPPIST-1 g".into(),
        ];
        let request = serde_json::json!({
            "system": {"catalog_system_star_count": 2},
            "star": {"measurements": {"age": {"display": "a reported upper limit of 1 billion years"}, "mass": {}, "radius": {}, "temperature": {}}},
            "publishable_comparisons": ["L 98-59 b's year is shorter than Earth's roughly 365-day year."]
        });
        ValidationContext {
            claims: ClaimContext::from_request(&request),
            planet_names,
            spectral_label: None,
            licensed_tokens: BTreeSet::new(),
            licensed_phrases: BTreeSet::new(),
        }
    }

    #[test]
    fn earth_year_comparisons_are_omitted_even_when_old_requests_license_them() {
        let ctx = claim_context();
        for sentence in [
            "**L 98-59 b**'s year is shorter than Earth's roughly 365-day year.",
            "Its year is shorter than Earth's year.",
            "The period is about 3 Earth years.",
            "Compared with an Earth-year, this is short.",
        ] {
            assert!(!claim_violations(sentence, &ctx).is_empty(), "{sentence}");
        }
        for sentence in [
            "Its orbital period is **about 365 days**.",
            "Its year lasts **about 2000 days**.",
            "It has the shortest year among the listed planets.",
            "Its radius is larger than Earth's.",
        ] {
            assert!(claim_violations(sentence, &ctx).is_empty(), "{sentence}");
        }
    }

    #[test]
    fn stellar_measurements_cannot_be_assigned_to_a_multi_star_system() {
        let ctx = claim_context();
        for sentence in [
            "The system also has a reported age upper limit of 1 billion years.",
            "The system's mass is about 0.95 solar masses.",
            "The radius of the whole system is about 0.85 solar radii.",
            "The system-wide temperature is about 5310 K.",
            "It has a minimum mass of about 0.95 solar masses.",
        ] {
            assert!(!claim_violations(sentence, &ctx).is_empty(), "{sentence}");
        }
        for sentence in [
            "HD 41004 A has an age upper limit of 1 billion years.",
            "The star's mass is about 0.95 solar masses.",
            "The system contains two stars. The host star has a radius of about 0.85 solar radii.",
            "The system is about 120 light-years away.",
        ] {
            assert!(claim_violations(sentence, &ctx).is_empty(), "{sentence}");
        }
        let mut single = ctx;
        single.claims = ClaimContext::default();
        assert!(
            claim_violations("The system's age is 1 billion years.", &single)
                .is_empty()
        );
    }

    #[test]
    fn normalization_and_gate_handle_complete_measurements_across_strong_boundaries()
     {
        for fragment in [
            "about **0.198 g/cm**³",
            "**about 0.198** g/cm³",
            "about 0.198 g/cm³",
            "**about** 0.198 **g/cm³**",
        ] {
            let article = format!("# T\n\nIts density is {fragment}.");
            assert!(unbolded_measurements(&article, None).is_err(), "{fragment}");
            let normalized = normalize_article(&article, None, &[]);
            assert!(normalized.contains("**0.198 g/cm³**"), "{normalized}");
            assert!(unbolded_measurements(&normalized, None).is_ok());
            assert_eq!(normalize_article(&normalized, None, &[]), normalized);
        }
    }

    #[test]
    fn complete_planet_coverage_is_required_before_installation() {
        let ctx = claim_context();
        let errors = validate_article("# Host\n\nA star.", &ctx).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error == "missing requested planet: L 98-59 e")
        );
    }

    #[test]
    fn removes_quantity_about_without_changing_bounds_or_ordinary_prose() {
        let input = "# T\n\nIts minimum mass is **about 5 Earth masses**, with an upper limit of **10 Earth masses** and a lower age limit of **4 billion years**. It orbits about every **2.5 days**. A radius of about four Earth radii is about half the Sun's radius. Its mass is about 1.06 times that of the Sun. This says nothing about its composition or about 51 Peg. A discussion about one star.";
        let expected = "# T\n\nIts minimum mass is **5 Earth masses**, with an upper limit of **10 Earth masses** and a lower age limit of **4 billion years**. It orbits every **2.5 days**. A radius of four Earth radii is half the Sun's radius. Its mass is 1.06 times that of the Sun. This says nothing about its composition or about 51 Peg. A discussion about one star.";
        let actual = remove_measurement_about(input);
        assert_eq!(actual, expected);
        assert_eq!(numeric_tokens(&actual), numeric_tokens(input));
        assert_eq!(remove_measurement_about(&actual), actual);
    }

    #[test]
    fn removes_about_from_ranges_lists_and_written_units() {
        for (input, expected) in [
            ("about 1,**350 light-years**", "1,**350 light-years**"),
            (
                "about 3.42 grams per cubic centimeter",
                "3.42 grams per cubic centimeter",
            ),
            (
                "between about 2.5 and **4 times Earth's**",
                "between 2.5 and **4 times Earth's**",
            ),
            (
                "about 1.5 to **2.5 times Earth's** radius",
                "1.5 to **2.5 times Earth's** radius",
            ),
            (
                "about 4.08, 11.4, and **6.83 days**",
                "4.08, 11.4, and **6.83 days**",
            ),
            (
                "read about 2MASS J16262785-2625152",
                "read about 2MASS J16262785-2625152",
            ),
        ] {
            assert_eq!(remove_measurement_about(input), expected, "{input}");
        }
    }
}
