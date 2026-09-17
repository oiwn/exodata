//! Native generation of saved requests, with current per-system results only.
use super::client::{Client, MODEL, Outcome};
use super::validate;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Instant, SystemTime},
};

const FINGERPRINT_VERSION: u32 = 10;

mod input;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Options {
    pub input_dir: PathBuf,
    pub hostnames: Vec<String>,
    pub system_prompt: PathBuf,
    pub repair_prompt: PathBuf,
    pub concurrency: usize,
    pub max_tokens: u32,
    pub force: bool,
    /// Select only systems whose latest attempt failed (fail.toml present)
    /// and retry them even when a matching fingerprint would otherwise skip.
    pub failed: bool,
    /// Variant label: writes description_<label>.md and friends instead
    /// of the served description.md, for experiments and A/B runs.
    pub label: Option<String>,
}

/// Validate a variant label for filename use.
pub(crate) fn validate_label(label: &str) -> Result<()> {
    OutputNames::validate_label(label)
}

/// Output file names for one run, honoring the variant label.
#[derive(Clone)]
struct OutputNames {
    description: String,
    metadata: String,
    draft: String,
    fail: String,
}

impl OutputNames {
    fn new(label: Option<&str>) -> Result<Self> {
        if let Some(label) = label {
            Self::validate_label(label)?;
            Ok(Self {
                description: format!("description_{label}.md"),
                metadata: format!("metadata_{label}.toml"),
                draft: format!("draft_{label}.md"),
                fail: format!("fail_{label}.toml"),
            })
        } else {
            Ok(Self {
                description: "description.md".into(),
                metadata: "metadata.toml".into(),
                draft: "draft.md".into(),
                fail: "fail.toml".into(),
            })
        }
    }

    /// Labels become filename suffixes: nonempty alphanumeric/dashes.
    pub(crate) fn validate_label(label: &str) -> Result<()> {
        if label.is_empty()
            || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            bail!("label must be nonempty alphanumeric/dashes");
        }
        Ok(())
    }
}

struct Job {
    directory: PathBuf,
    hostname: String,
    input: String,
    fingerprint: String,
    source_date: Option<String>,
    validation: validate::ValidationContext,
    output: OutputNames,
}

/// One writing call, optionally followed by one repair.
struct AttemptOutcome {
    outcome: Outcome,
    attempts: u32,
    draft_usage: super::usage::Usage,
    repair_usage: Option<super::usage::Usage>,
    failed_stage: Option<&'static str>,
    final_errors: Option<Vec<String>>,
    recovered: Vec<Vec<String>>,
}

impl AttemptOutcome {
    fn usage(&self) -> super::usage::Usage {
        self.repair_usage
            .map_or(self.draft_usage, |repair| self.draft_usage.add(repair))
    }
}

async fn generate_system(
    client: &Client,
    job: &Job,
    drafter_prompt: &str,
    repair_prompt: &str,
    max_tokens: u32,
) -> Result<AttemptOutcome> {
    let mut outcome = client
        .generate(&job.input, Some(drafter_prompt), max_tokens, false)
        .await;
    let draft_usage = super::usage::Usage::read(
        &outcome.response.as_ref().unwrap_or(&Value::Null)["usage"],
    );
    let mut candidate = outcome
        .report
        .as_ref()
        .and_then(|r| r["text"].as_str())
        .map(|text| {
            validate::normalize_article(
                text,
                job.validation.spectral_label.as_deref(),
                &job.validation.planet_names,
            )
        });
    let mut errors = candidate
        .as_ref()
        .and_then(|text| validate::validate_article(text, &job.validation).err());
    let mut recovered = Vec::new();
    let mut repair_usage = None;
    let mut failed_stage =
        (outcome.error.is_some() || errors.is_some()).then_some("draft");
    if let Some(text) = &candidate {
        // Keep the normalized first response for review, even when invalid.
        fs::write(job.directory.join(&job.output.draft), text)
            .with_context(|| format!("Cannot save draft for {}", job.hostname))?;
    }
    if let Some(violations) = &errors {
        let repair_input = serde_json::to_string(&json!({
            "facts": serde_json::from_str::<Value>(&job.input)?,
            "article": candidate.as_deref().unwrap_or_default(),
            "violations": violations,
        }))?;
        recovered.push(violations.clone());
        let draft_elapsed = outcome.elapsed_ms;
        outcome = client
            .generate(&repair_input, Some(repair_prompt), max_tokens, false)
            .await;
        repair_usage = Some(super::usage::Usage::read(
            &outcome.response.as_ref().unwrap_or(&Value::Null)["usage"],
        ));
        outcome.elapsed_ms += draft_elapsed;
        candidate = outcome
            .report
            .as_ref()
            .and_then(|r| r["text"].as_str())
            .map(|text| {
                validate::normalize_article(
                    text,
                    job.validation.spectral_label.as_deref(),
                    &job.validation.planet_names,
                )
            });
        errors = candidate.as_ref().and_then(|text| {
            validate::validate_article(text, &job.validation).err()
        });
        failed_stage =
            (outcome.error.is_some() || errors.is_some()).then_some("repair");
        if let Some(errors) = &errors {
            recovered.push(errors.clone());
        }
    }
    if let (Some(text), Some(report)) = (candidate, &mut outcome.report) {
        report["text"] = json!(text);
    }
    Ok(AttemptOutcome {
        outcome,
        attempts: 1 + u32::from(repair_usage.is_some()),
        draft_usage,
        repair_usage,
        failed_stage,
        final_errors: errors,
        recovered,
    })
}

pub fn columns() -> Vec<String> {
    [
        "hostname",
        "status",
        "reason",
        "model",
        "finish_reason",
        "prompt_tokens",
        "completion_tokens",
        "total_tokens",
        "usage_complete",
        "elapsed_ms",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// Collect numeric tokens licensed for reader-facing prose: display text,
/// names, spectral labels, approved comparisons, included guide values,
/// discovery years, and system counts. Raw audit values, errors, and the
/// instruction/source/silent sections do not license anything.
fn collect_licensed(value: &Value, licensed: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                match key.as_str() {
                    "display" | "spectral_type" => {
                        if let Some(text) = item.as_str() {
                            licensed.extend(validate::numeric_tokens(text));
                        }
                    }
                    "discovery_year"
                    | "matching_host_planet_count"
                    | "catalog_system_planet_count"
                    | "catalog_system_star_count" => {
                        if let Some(number) = item.as_u64() {
                            licensed.insert(number.to_string());
                        }
                    }
                    "request" | "source" | "silent_constraints"
                    | "schema_version" => {}
                    _ => collect_licensed(item, licensed),
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_licensed(item, licensed);
            }
        }
        Value::String(text) if !validate::is_earth_year_reference(text) => {
            licensed.extend(validate::numeric_tokens(text));
        }
        Value::String(_) => {}
        _ => {}
    }
}

fn settings(max_tokens: u32) -> Value {
    json!({"model": MODEL, "max_tokens": max_tokens, "thinking": "disabled", "stream": false,
        "request_timeout_seconds": 60, "retries": 0})
}

/// Phrases whose reader-facing use is licensed by prepared facts: the
/// circumbinary planet flag, a pulsar host kind plus the Pulsar Timing
/// method name, a very-young age class, and Sun-like wording only for a
/// supplied G-type spectral label.
fn licensed_phrases(
    request: &Value,
    spectral_label: &Option<String>,
) -> BTreeSet<String> {
    let mut phrases = BTreeSet::new();
    let planets = request["planets"].as_array();
    if request["star"]["host_kind"].as_str() == Some("pulsar") {
        phrases.insert("pulsar".to_owned());
    }
    if planets.is_some_and(|planets| {
        planets
            .iter()
            .any(|p| p["circumbinary"].as_bool() == Some(true))
    }) {
        phrases.insert("circumbinary".to_owned());
    }
    if planets.is_some_and(|planets| {
        planets
            .iter()
            .any(|p| p["discovery_method"].as_str() == Some("Pulsar Timing"))
    }) {
        phrases.insert("pulsar timing".to_owned());
    }
    if request["star"]["age_class"].as_str() == Some("very young") {
        phrases.insert("very young".to_owned());
    }
    if spectral_label
        .as_deref()
        .is_some_and(|label| label.trim_start().to_uppercase().starts_with('G'))
    {
        phrases.insert("sun-like".to_owned());
    }
    phrases
}

fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map.iter().collect();
            Value::Object(
                sorted
                    .into_iter()
                    .map(|(k, v)| (k.clone(), canonical(v)))
                    .collect(),
            )
        }
        Value::Array(values) => {
            Value::Array(values.iter().map(canonical).collect())
        }
        _ => value.clone(),
    }
}

fn fingerprint(
    request: &Value,
    prompt: &str,
    style_prompt: &str,
    max_tokens: u32,
) -> Result<String> {
    let input = canonical(&json!({"fingerprint_version": FINGERPRINT_VERSION,
        "request": request, "system_prompt": prompt,
        "style_prompt": style_prompt,
        "settings": settings(max_tokens)}));
    let digest = Sha256::digest(serde_json::to_vec(&input)?);
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(format!("sha256:{hex}"))
}

fn read_toml(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    Ok(serde_json::to_value(
        toml::from_str::<toml::Value>(&text)
            .with_context(|| format!("Invalid TOML in {}", path.display()))?,
    )?)
}

/// Merge optional hand-edited `notes.toml` into the request in memory:
/// `facts` entries become publishable comparisons (their numbers license
/// reader-facing use) and `guidance` entries become silent constraints.
/// The merged request feeds the fingerprint and the wire input, so
/// edited notes regenerate the system. Prepare and batch never write
/// this file.
fn merge_notes(directory: &Path, request: &mut Value) -> Result<()> {
    let path = directory.join("notes.toml");
    if !path.try_exists()? {
        return Ok(());
    }
    let notes = read_toml(&path)?;
    for (key, target) in [
        ("facts", "publishable_comparisons"),
        ("guidance", "silent_constraints"),
    ] {
        if let Some(items) = notes[key].as_array() {
            if !request[target].is_array() {
                request[target] = json!([]);
            }
            let list = request[target].as_array_mut().with_context(|| {
                format!("notes.toml: request {target} is not a list")
            })?;
            for item in items {
                let text = item.as_str().with_context(|| {
                    format!("notes.toml: {key} entries must be strings")
                })?;
                list.push(Value::String(text.to_owned()));
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn row(
    hostname: &str,
    status: &str,
    reason: &str,
    attempts: Option<u32>,
    outcome: Option<&Outcome>,
) -> Value {
    let response = outcome.and_then(|o| o.response.as_ref());
    json!({"hostname": hostname, "status": status, "reason": reason,
        "attempts": attempts,
        "model": response.and_then(|r|r["model"].as_str()),
        "finish_reason": response.and_then(|r|r["choices"][0]["finish_reason"].as_str()),
        "prompt_tokens": response.and_then(|r|r["usage"]["prompt_tokens"].as_u64()),
        "completion_tokens": response.and_then(|r|r["usage"]["completion_tokens"].as_u64()),
        "total_tokens": response.and_then(|r|r["usage"]["total_tokens"].as_u64()),
        "elapsed_ms": outcome.map(|o|o.elapsed_ms)})
}

/// Compact single-line outcome for the `lines` output format.
pub fn line(row: &Value) -> String {
    let hostname = row["hostname"].as_str().unwrap_or("?");
    let status = row["status"].as_str().unwrap_or("?");
    let attempts = row["attempts"].as_u64();
    let tokens = row["total_tokens"].as_u64().unwrap_or_default();
    let elapsed = row["elapsed_ms"].as_u64().unwrap_or_default();
    let tail = match row["reason"].as_str() {
        Some(reason) if !reason.is_empty() => truncate(reason, 80),
        _ => row["model"].as_str().unwrap_or_default().to_owned(),
    };
    format!(
        "{hostname:<26} {status:<9} {:>2} att {:>6} tok {:>7} {tail}",
        attempts
            .map(|a| a.to_string())
            .unwrap_or_else(|| "-".into()),
        super::short_tokens(tokens),
        super::short_elapsed_ms(elapsed),
    )
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        let cut: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

fn preflight(
    options: &Options,
    prompt: &str,
    style_prompt: &str,
) -> Result<(VecDeque<Job>, Vec<Value>)> {
    if options.concurrency == 0 || options.max_tokens == 0 {
        bail!("Concurrency and max_tokens must be positive");
    }
    if prompt.trim().is_empty() {
        bail!("System prompt must not be blank");
    }
    let output_names = OutputNames::new(options.label.as_deref())?;
    let filters: BTreeSet<_> =
        options.hostnames.iter().map(String::as_str).collect();
    let mut selected = BTreeMap::new();
    let mut prepared = 0;
    for entry in fs::read_dir(&options.input_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let directory = entry.path();
        let request_path = directory.join("request.toml");
        if !request_path.try_exists()? {
            continue;
        }
        let mut request = read_toml(&request_path)?;
        let hostname = request["system"]["hostname"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .with_context(|| {
                format!("{} has no system.hostname", request_path.display())
            })?
            .to_owned();
        if !filters.is_empty() && !filters.contains(hostname.as_str()) {
            continue;
        }
        if request["schema_version"].as_u64() != Some(1)
            || !request["planets"].is_array()
        {
            bail!("Unsupported prepared request schema for {hostname}");
        }
        if options.failed && !directory.join(&output_names.fail).try_exists()? {
            prepared += 1;
            continue;
        }
        merge_notes(&directory, &mut request)?;
        let evidence_path = directory.join("evidence.json");
        let evidence: Value = serde_json::from_str(&fs::read_to_string(&evidence_path)
            .with_context(||format!("Missing evidence for {hostname}; run descriptions prepare first"))?)?;
        if evidence["hostname"].as_str() != Some(&hostname) {
            bail!("Evidence hostname mismatch for {hostname}");
        }
        for name in [".prepare", ".generate", ".fail-write"] {
            if directory.join(name).try_exists()? {
                bail!(
                    "Pending operation/recovery at {}",
                    directory.join(name).display()
                );
            }
        }
        let hash =
            fingerprint(&request, prompt, style_prompt, options.max_tokens)?;
        let source_date =
            request["source"]["source_date"].as_str().map(str::to_owned);
        // Project compact facts after notes merging; keep audit data off the wire.
        let input = serde_json::to_string(&input::project(&request))?;
        let planet_names: Vec<String> = request["planets"]
            .as_array()
            .map(|planets| {
                planets
                    .iter()
                    .filter_map(|planet| {
                        planet["name"].as_str().map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let spectral_label =
            request["star"]["spectral_type"].as_str().map(str::to_owned);
        let mut licensed_tokens = validate::numeric_tokens(&hostname);
        for name in &planet_names {
            licensed_tokens.extend(validate::numeric_tokens(name));
        }
        if let Some(label) = &spectral_label {
            licensed_tokens.extend(validate::numeric_tokens(label));
        }
        collect_licensed(&request, &mut licensed_tokens);
        let licensed_phrases = licensed_phrases(&request, &spectral_label);
        let validation = validate::ValidationContext {
            claims: validate::ClaimContext::from_request(&request),
            planet_names,
            spectral_label,
            licensed_tokens,
            licensed_phrases,
        };
        if selected
            .insert(
                hostname.clone(),
                Job {
                    directory,
                    hostname,
                    input,
                    fingerprint: hash,
                    source_date,
                    validation,
                    output: output_names.clone(),
                },
            )
            .is_some()
        {
            bail!("Duplicate prepared hostname");
        }
    }
    for name in &filters {
        if !selected.contains_key(*name) {
            if options.failed {
                bail!("No failed system for {name}");
            }
            bail!("No prepared request for {name}");
        }
    }
    if selected.is_empty() {
        if options.failed && prepared > 0 {
            return Ok((VecDeque::new(), Vec::new()));
        }
        bail!("No prepared requests found");
    }
    let mut jobs = VecDeque::new();
    let mut rows = Vec::new();
    for (_, job) in selected {
        let metadata_path = job.directory.join(&job.output.metadata);
        let metadata = if metadata_path.try_exists()? {
            Some(read_toml(&metadata_path)?)
        } else {
            None
        };
        if metadata
            .as_ref()
            .is_some_and(|m| m["hostname"].as_str() != Some(&job.hostname))
        {
            bail!("Metadata hostname mismatch for {}", job.hostname);
        }
        let description_path = job.directory.join(&job.output.description);
        let description = if description_path.try_exists()? {
            fs::read_to_string(&description_path)?
        } else {
            String::new()
        };
        let matches = metadata.as_ref().is_some_and(|m| {
            m["fingerprint"].as_str() == Some(&job.fingerprint)
                && m["fingerprint_version"].as_u64()
                    == Some(FINGERPRINT_VERSION as u64)
        });
        let has_failure = job.directory.join(&job.output.fail).try_exists()?;
        if !options.force
            && !(options.failed && has_failure)
            && matches
            && !description.trim().is_empty()
        {
            rows.push(row(
                &job.hostname,
                "skipped",
                "generation inputs unchanged",
                None,
                None,
            ));
        } else {
            // Verify that per-system staging is writable before starting paid work.
            let staging = job.directory.join(".generate");
            fs::create_dir(&staging).with_context(|| {
                format!("Cannot write generation files for {}", job.hostname)
            })?;
            fs::remove_dir(&staging)?;
            for name in [
                job.output.metadata.clone(),
                job.output.description.clone(),
                job.output.fail.clone(),
            ] {
                let path = job.directory.join(name);
                if path.try_exists()? && !path.is_file() {
                    bail!("{} is not a file", path.display());
                }
            }
            jobs.push_back(job);
        }
    }
    Ok((jobs, rows))
}

fn now() -> String {
    chrono::DateTime::<chrono::Utc>::from(SystemTime::now()).to_rfc3339()
}

fn record(job: &Job, max_tokens: u32, result: &AttemptOutcome) -> Value {
    let AttemptOutcome {
        outcome,
        attempts,
        draft_usage,
        repair_usage,
        recovered,
        ..
    } = result;
    let mut doc = json!({"hostname": job.hostname, "fingerprint": job.fingerprint,
        "fingerprint_version": FINGERPRINT_VERSION, "settings": settings(max_tokens), "elapsed_ms": outcome.elapsed_ms,
        "attempts": attempts,
        "stages": {"draft": {"attempts": 1}}});
    draft_usage.write(&mut doc["stages"]["draft"]);
    if let Some(usage) = repair_usage {
        doc["stages"]["repair"] = json!({"attempts": 1});
        usage.write(&mut doc["stages"]["repair"]);
    }
    if !recovered.is_empty() {
        doc["validation_recovered"] =
            json!(recovered.iter().flatten().cloned().collect::<Vec<_>>());
    }
    if let Some(status) = outcome.status {
        doc["http_status"] = json!(status);
    }
    let report = row(&job.hostname, "", "", None, Some(outcome));
    for key in [
        "model",
        "finish_reason",
        "prompt_tokens",
        "completion_tokens",
        "total_tokens",
    ] {
        if !report[key].is_null() {
            doc[key] = report[key].clone();
        }
    }
    result.usage().write(&mut doc);
    doc
}

fn save_failure(
    job: &Job,
    max_tokens: u32,
    result: &AttemptOutcome,
    reason: &str,
) -> Result<()> {
    let mut doc = record(job, max_tokens, result);
    doc["failed_at"] = json!(now());
    doc["error"] = json!(reason);
    if let Some(body) = &result.outcome.body {
        doc["response_body"] = json!(body);
    }
    let staged = job.directory.join(".fail-write");
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged)?;
    file.write_all(toml::to_string_pretty(&doc)?.as_bytes())?;
    drop(file);
    fs::rename(&staged, job.directory.join(&job.output.fail))?;
    Ok(())
}

fn save_success(
    job: &Job,
    max_tokens: u32,
    result: &AttemptOutcome,
) -> Result<()> {
    let report = result
        .outcome
        .report
        .as_ref()
        .context("Missing successful response")?;
    let text = report["text"].as_str().context("Missing description")?;
    let mut doc = record(job, max_tokens, result);
    doc["generated_at"] = json!(now());
    if let Some(date) = &job.source_date {
        doc["source_date"] = json!(date);
    }
    let metadata = toml::to_string_pretty(&doc)?;
    super::prepare::store_pair(
        &job.directory,
        ".generate",
        [
            (&job.output.description, text),
            (&job.output.metadata, &metadata),
        ],
        || Ok(()),
        |from, to| fs::rename(from, to),
    )?;
    match fs::remove_file(job.directory.join(&job.output.fail)) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => {
            return Err(e).context(
                "Result saved but the previous fail file could not be removed",
            );
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn execute(
    mut jobs: VecDeque<Job>,
    mut rows: Vec<Value>,
    client: Client,
    prompt: String,
    style_prompt: String,
    concurrency: usize,
    max_tokens: u32,
) -> Result<Vec<Value>> {
    let prompt = Arc::new(prompt);
    let style_prompt = Arc::new(style_prompt);
    let mut tasks = tokio::task::JoinSet::new();
    let mut halted = false;
    while !jobs.is_empty() || !tasks.is_empty() {
        while !halted && tasks.len() < concurrency {
            let Some(job) = jobs.pop_front() else {
                break;
            };
            let client = client.clone();
            let prompt = prompt.clone();
            let style_prompt = style_prompt.clone();
            eprintln!("Generating {}", job.hostname);
            tasks.spawn(async move {
                let result = generate_system(
                    &client,
                    &job,
                    &prompt,
                    &style_prompt,
                    max_tokens,
                )
                .await;
                (job, result)
            });
        }
        let Some(finished) = tasks.join_next().await else {
            break;
        };
        let (job, result) = finished.context("Generation worker failed")?;
        let result = result.inspect_err(|error| {
            eprintln!("{}: draft storage failure: {error:#}", job.hostname);
        })?;
        let AttemptOutcome {
            outcome,
            attempts,
            failed_stage,
            final_errors,
            ..
        } = &result;
        let attempts = *attempts;
        let final_errors = final_errors.clone();
        let failed_stage = *failed_stage;
        halted |= outcome.stop_batch();
        let mut reason = outcome.error.clone().unwrap_or_default();
        let mut status = "generated";
        if outcome.report.is_some() && final_errors.is_none() {
            if let Err(error) = save_success(&job, max_tokens, &result) {
                halted = true;
                reason = format!("Storage failure: {error:#}");
                status = "failed";
            }
        } else {
            status = "failed";
            if let (Some(stage), Some(errors)) = (failed_stage, &final_errors) {
                reason = format!(
                    "{stage} validation failed after {attempts} total calls: {}",
                    errors.join("; ")
                );
            }
        }
        if status == "failed"
            && let Err(error) = save_failure(&job, max_tokens, &result, &reason)
        {
            halted = true;
            reason.push_str(&format!("; cannot save fail.toml: {error:#}"));
        }
        eprintln!(
            "{}: {status}{}{}",
            job.hostname,
            if attempts > 1 && status == "generated" {
                format!(" (validated after {attempts} attempts)")
            } else {
                String::new()
            },
            if reason.is_empty() {
                String::new()
            } else {
                format!(" ({reason})")
            }
        );
        let mut output = row(
            &job.hostname,
            status,
            &reason,
            Some(attempts),
            Some(outcome),
        );
        result.usage().write(&mut output);
        rows.push(output);
    }
    rows.extend(jobs.iter().map(|job| {
        row(
            &job.hostname,
            "not_started",
            "batch stopped after an account, rate-limit, or storage failure",
            None,
            None,
        )
    }));
    rows.sort_by(|a, b| a["hostname"].as_str().cmp(&b["hostname"].as_str()));
    Ok(rows)
}

pub fn run(options: &Options) -> Result<Vec<Value>> {
    let started = Instant::now();
    let prompt = fs::read_to_string(&options.system_prompt)?;
    let style_prompt = fs::read_to_string(&options.repair_prompt)?;
    let (jobs, mut rows) = preflight(options, &prompt, &style_prompt)?;
    if options.failed && jobs.is_empty() && rows.is_empty() {
        eprintln!("No failed systems");
        return Ok(rows);
    }
    if !jobs.is_empty() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        rows = runtime.block_on(async {
            execute(
                jobs,
                rows,
                Client::from_env()?,
                prompt,
                style_prompt,
                options.concurrency,
                options.max_tokens,
            )
            .await
        })?;
    }
    let total: u64 = rows.iter().filter_map(|r| r["total_tokens"].as_u64()).sum();
    eprintln!(
        "{} systems; {} generated, {} skipped, {} failed/not started; reported tokens: {total}; wall time: {:.2}s",
        rows.len(),
        rows.iter().filter(|r| r["status"] == "generated").count(),
        rows.iter().filter(|r| r["status"] == "skipped").count(),
        rows.iter()
            .filter(|r| r["status"] == "failed" || r["status"] == "not_started")
            .count(),
        started.elapsed().as_secs_f64()
    );
    Ok(rows)
}
