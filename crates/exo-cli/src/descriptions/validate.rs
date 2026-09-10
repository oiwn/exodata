//! Deterministic Markdown gate for generated stellar-host descriptions.
use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::BTreeSet;

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
}

/// Full deterministic gate chain: Markdown structure, bolded names and
/// measurements, punctuation, numeric allowlist, and banned phrasings.
pub(super) fn validate_article(
    markdown: &str,
    ctx: &ValidationContext,
) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
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
    let mut parser = Parser::new_ext(markdown, Options::empty()).peekable();
    let _ = parser.next();
    let mut in_title = true;
    let mut strong_depth = 0usize;
    for event in parser {
        match event {
            Event::Text(text) => {
                if !in_title && strong_depth == 0 {
                    if let Some(phrase) = number_with_unit(&text) {
                        violations.push(format!(
                            "measurement phrase '{phrase}' must be bolded"
                        ));
                    }
                    if let Some(label) = spectral_label {
                        if text.contains(label) {
                            violations.push(format!(
                                "spectral label '{label}' must be bolded"
                            ));
                        }
                    }
                }
            }
            Event::Start(Tag::Strong) => strong_depth += 1,
            Event::End(TagEnd::Strong) => {
                strong_depth = strong_depth.saturating_sub(1)
            }
            Event::End(TagEnd::Heading(_)) if in_title => in_title = false,
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
    "g/cm",
    "times jupiter",
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
                    unit_len = unit.len();
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
            return Some(phrase.trim().to_owned());
        }
    }
    None
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
];

const BANNED_WORDS: &[&str] = &["inner", "outer"];

pub(super) fn banned_patterns(
    markdown: &str,
    licensed: &BTreeSet<String>,
) -> Result<(), Vec<String>> {
    let mut text = markdown.replace("**", "").to_lowercase();
    for phrase in licensed {
        let phrase = phrase.to_lowercase();
        if !phrase.is_empty() {
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

/// Editorial fact-preservation gate: the edited article must contain
/// exactly the same numeric tokens as the draft (no new numbers, none
/// lost), and every planet name the draft mentioned must still appear.
/// Bold markers are ignored while scanning.
pub(super) fn preserves_facts(
    draft: &str,
    edited: &str,
    planet_names: &[String],
) -> Result<(), Vec<String>> {
    let before = numeric_tokens(draft);
    let after = numeric_tokens(edited);
    let mut violations = Vec::new();
    let missing: Vec<_> = before.difference(&after).collect();
    let added: Vec<_> = after.difference(&before).collect();
    if !missing.is_empty() {
        violations.push(format!(
            "value(s) lost in editing: {}",
            missing
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !added.is_empty() {
        violations.push(format!(
            "value(s) introduced in editing: {}",
            added
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for name in planet_names {
        if draft.contains(name.as_str()) && !edited.contains(name.as_str()) {
            violations.push(format!("planet name {name} no longer appears"));
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
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
        assert!(banned_patterns("# T\n\na young star", &licensed).is_err());
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
    fn fact_preservation_requires_identical_numbers_and_names() {
        let draft = "# T\n\n**TRAPPIST-1 b** orbits in **about 1.51 days**.";
        assert!(preserves_facts(draft, draft, &names()).is_ok());
        assert!(
            preserves_facts(
                draft,
                "# T\n\n**TRAPPIST-1 b** circles in **about 1.52 days**.",
                &names()
            )
            .is_err()
        );
        let errors = preserves_facts(
            draft,
            "# T\n\n**TRAPPIST-1 c** orbits in **about 1.5 days** and **5** \
             more.",
            &names(),
        )
        .unwrap_err();
        assert!(errors.iter().any(|v| v.contains("lost in editing: 1.51")));
        assert!(errors.iter().any(|v| v.contains("introduced in editing")));
        assert!(
            errors
                .iter()
                .any(|v| v.contains("TRAPPIST-1 b no longer appears"))
        );
        assert!(preserves_facts(
            draft,
            "# T\n\n**TRAPPIST-1 b** takes **1.51** days (0.41 Earth-**years**).",
            &names()
        )
        .is_err());
    }
}
