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

const FINGERPRINT_VERSION: u32 = 3;

#[cfg(test)]
mod tests;

pub struct Options {
    pub input_dir: PathBuf,
    pub hostnames: Vec<String>,
    pub system_prompt: PathBuf,
    pub editor_prompt: PathBuf,
    pub critic_prompt: PathBuf,
    pub concurrency: usize,
    pub max_tokens: u32,
    pub force: bool,
}

struct Job {
    directory: PathBuf,
    hostname: String,
    input: String,
    fingerprint: String,
    source_date: Option<String>,
    validation: validate::ValidationContext,
}

/// Result of one system's draft-edit-critic cycle.
struct AttemptOutcome {
    outcome: Outcome,
    attempts: u32,
    draft_attempts: u32,
    edit_attempts: u32,
    draft_usage: (u64, u64, u64),
    edit_usage: (u64, u64, u64),
    critic_findings: Vec<String>,
    critic_corrected: bool,
    critic_unparseable: bool,
    critic_usage: (u64, u64, u64),
    /// Stage that failed validation, when validation never passed.
    failed_stage: Option<&'static str>,
    /// Violations from the final attempt when validation never passed.
    final_errors: Option<Vec<String>>,
    /// Violations from every rejected attempt, oldest first.
    recovered: Vec<Vec<String>>,
}

const MAX_ATTEMPTS: u32 = 3;

/// Overwrite reported usage in both the parsed report and the raw response.
fn set_usage(outcome: &mut Outcome, prompt: u64, completion: u64, total: u64) {
    if let Some(report) = &mut outcome.report {
        report["prompt_tokens"] = json!(prompt);
        report["completion_tokens"] = json!(completion);
        report["total_tokens"] = json!(total);
    }
    if let Some(response) = &mut outcome.response {
        response["usage"]["prompt_tokens"] = json!(prompt);
        response["usage"]["completion_tokens"] = json!(completion);
        response["usage"]["total_tokens"] = json!(total);
    }
}

/// One stage: up to MAX_ATTEMPTS validated calls with feedback retries.
struct StageOutcome {
    outcome: Outcome,
    attempts: u32,
    recovered: Vec<Vec<String>>,
    passed: bool,
    usage: (u64, u64, u64),
    text: String,
}

async fn run_stage(
    client: &Client,
    prompt: &str,
    input: &str,
    validation: &validate::ValidationContext,
    max_tokens: u32,
    extra_gate: Option<&(dyn Fn(&str) -> Result<(), Vec<String>> + Send + Sync)>,
) -> StageOutcome {
    let mut recovered: Vec<Vec<String>> = Vec::new();
    let mut usage = (0u64, 0u64, 0u64);
    let mut elapsed = 0u128;
    let mut passed = false;
    let mut text = String::new();
    let mut outcome = None;
    for attempt in 1..=MAX_ATTEMPTS {
        let input = if attempt == 1 {
            input.to_owned()
        } else {
            let feedback = match recovered.last() {
                Some(errors) => errors
                    .iter()
                    .map(|error| format!("- {error}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                None => String::new(),
            };
            format!(
                "{input}\n\n# Formatting feedback\n\nThe previous attempt \
                 violated these rules:\n{feedback}\nReturn the complete \
                 corrected article."
            )
        };
        let result = client.generate(&input, Some(prompt), max_tokens).await;
        elapsed += result.elapsed_ms;
        if let Some(report) = &result.report {
            usage.0 += report["prompt_tokens"].as_u64().unwrap_or(0);
            usage.1 += report["completion_tokens"].as_u64().unwrap_or(0);
            usage.2 += report["total_tokens"].as_u64().unwrap_or(0);
        }
        let transport_failed = result.report.is_none();
        let candidate = result
            .report
            .as_ref()
            .and_then(|report| report["text"].as_str())
            .unwrap_or_default()
            .to_owned();
        let errors = if transport_failed {
            None
        } else {
            Some(validate::validate_article(&candidate, validation).and_then(
                |_| match extra_gate {
                    Some(gate) => gate(&candidate),
                    None => Ok(()),
                },
            ))
        };
        let invalid = matches!(&errors, Some(Err(_)));
        if let Some(Err(list)) = errors {
            recovered.push(list);
        }
        let mut result = result;
        if result.report.is_some() {
            set_usage(&mut result, usage.0, usage.1, usage.2);
        }
        result.elapsed_ms = elapsed;
        if !invalid {
            passed = !transport_failed;
            text = candidate;
        }
        outcome = Some(result);
        if !invalid {
            break;
        }
    }
    let outcome = outcome.expect("at least one attempt always runs");
    let attempts = (recovered.len() as u32 + 1).min(MAX_ATTEMPTS);
    StageOutcome {
        outcome,
        attempts,
        recovered,
        passed,
        usage,
        text,
    }
}

async fn generate_two_stage(
    client: &Client,
    job: &Job,
    drafter_prompt: &str,
    editor_prompt: &str,
    critic_prompt: &str,
    max_tokens: u32,
) -> Result<AttemptOutcome> {
    let draft = run_stage(
        client,
        drafter_prompt,
        &job.input,
        &job.validation,
        max_tokens,
        None,
    )
    .await;
    if !draft.passed {
        return Ok(AttemptOutcome {
            failed_stage: Some("draft"),
            final_errors: draft.recovered.last().cloned(),
            attempts: draft.attempts,
            draft_attempts: draft.attempts,
            edit_attempts: 0,
            draft_usage: draft.usage,
            edit_usage: (0, 0, 0),
            critic_findings: Vec::new(),
            critic_corrected: false,
            critic_unparseable: false,
            critic_usage: (0, 0, 0),
            recovered: draft.recovered,
            outcome: draft.outcome,
        });
    }
    fs::write(job.directory.join("draft.md"), &draft.text)
        .with_context(|| format!("Cannot save draft for {}", job.hostname))?;
    let draft_text = draft.text.clone();
    let editor_input = format!(
        "{}\n\n# Source evidence and instructions\n\n{}",
        draft.text, job.input
    );
    let editor = run_stage(
        client,
        editor_prompt,
        &editor_input,
        &job.validation,
        max_tokens,
        Some(&|edited| {
            validate::preserves_facts(
                &draft_text,
                edited,
                &job.validation.planet_names,
            )
        }),
    )
    .await;
    if !editor.passed {
        return Ok(AttemptOutcome {
            failed_stage: Some("editor"),
            final_errors: editor.recovered.last().cloned(),
            attempts: draft.attempts + editor.attempts,
            draft_attempts: draft.attempts,
            edit_attempts: editor.attempts,
            draft_usage: draft.usage,
            edit_usage: editor.usage,
            critic_findings: Vec::new(),
            critic_corrected: false,
            critic_unparseable: false,
            critic_usage: (0, 0, 0),
            recovered: [draft.recovered, editor.recovered].concat(),
            outcome: editor.outcome,
        });
    }
    let editor_attempts = editor.attempts;
    let editor_usage = editor.usage;
    let editor_recovered = editor.recovered.clone();
    let article = editor.text.clone();
    let critic = run_critic(
        client,
        critic_prompt,
        &article,
        &job.input,
        max_tokens.min(512),
    )
    .await;
    let mut critic_corrected = false;
    let final_stage = if !critic.findings.is_empty() {
        let correction_input = format!(
            "{article}\n\n# Verifier findings\n\nThe article contains \
             these violations:\n{}\nReturn the complete corrected article. \
             Delete or rephrase the offending passages without changing any \
             numbers.",
            critic.findings.join("\n")
        );
        let article_text = article.clone();
        let correction = run_stage(
            client,
            editor_prompt,
            &correction_input,
            &job.validation,
            max_tokens,
            Some(&|edited| {
                validate::preserves_facts(
                    &article_text,
                    edited,
                    &job.validation.planet_names,
                )
            }),
        )
        .await;
        if !correction.passed {
            return Ok(AttemptOutcome {
                failed_stage: Some("critic"),
                final_errors: Some(critic.findings.clone()),
                attempts: draft.attempts + editor_attempts + correction.attempts,
                draft_attempts: draft.attempts,
                edit_attempts: editor_attempts + correction.attempts,
                draft_usage: draft.usage,
                edit_usage: editor_usage,
                critic_findings: critic.findings,
                critic_corrected: false,
                critic_unparseable: critic.unparseable,
                critic_usage: critic.usage,
                recovered: [
                    draft.recovered,
                    editor_recovered,
                    correction.recovered,
                ]
                .concat(),
                outcome: correction.outcome,
            });
        }
        critic_corrected = true;
        correction
    } else {
        editor
    };
    let mut final_stage = final_stage;
    // final_stage.usage is the editor's (or the correction's) usage alone,
    // so the totals add each stage exactly once.
    let totals = (
        draft.usage.0 + critic.usage.0 + final_stage.usage.0,
        draft.usage.1 + critic.usage.1 + final_stage.usage.1,
        draft.usage.2 + critic.usage.2 + final_stage.usage.2,
    );
    set_usage(&mut final_stage.outcome, totals.0, totals.1, totals.2);
    final_stage.outcome.elapsed_ms +=
        draft.outcome.elapsed_ms + critic.elapsed_ms;
    Ok(AttemptOutcome {
        failed_stage: None,
        final_errors: None,
        attempts: draft.attempts
            + editor_attempts
            + if critic_corrected {
                final_stage.attempts
            } else {
                0
            },
        draft_attempts: draft.attempts,
        edit_attempts: editor_attempts
            + if critic_corrected {
                final_stage.attempts
            } else {
                0
            },
        draft_usage: draft.usage,
        edit_usage: final_stage.usage,
        critic_findings: critic.findings,
        critic_corrected,
        critic_unparseable: critic.unparseable,
        critic_usage: critic.usage,
        recovered: if critic_corrected {
            [draft.recovered, editor_recovered, final_stage.recovered].concat()
        } else {
            [draft.recovered, editor_recovered].concat()
        },
        outcome: final_stage.outcome,
    })
}

/// One verifier call (plus a single parse retry). Unparseable responses
/// fail open: the article stands and the outcome records the problem.
struct CriticOutcome {
    findings: Vec<String>,
    unparseable: bool,
    usage: (u64, u64, u64),
    elapsed_ms: u128,
}

async fn run_critic(
    client: &Client,
    critic_prompt: &str,
    article: &str,
    job_input: &str,
    max_tokens: u32,
) -> CriticOutcome {
    let input = format!("{article}\n\n# Source facts\n\n{job_input}");
    let mut usage = (0u64, 0u64, 0u64);
    let mut elapsed = 0u128;
    let mut findings = Vec::new();
    let mut unparseable = false;
    for attempt in 0..2 {
        let result = client
            .generate(&input, Some(critic_prompt), max_tokens)
            .await;
        elapsed += result.elapsed_ms;
        if let Some(report) = &result.report {
            usage.0 += report["prompt_tokens"].as_u64().unwrap_or(0);
            usage.1 += report["completion_tokens"].as_u64().unwrap_or(0);
            usage.2 += report["total_tokens"].as_u64().unwrap_or(0);
            let text = report["text"].as_str().unwrap_or_default();
            match parse_findings(text) {
                Some(list) => {
                    findings = list;
                    return CriticOutcome {
                        findings,
                        unparseable: false,
                        usage,
                        elapsed_ms: elapsed,
                    };
                }
                None => unparseable = true,
            }
        } else {
            // Transport failure: the critic is advisory; accept the article.
            break;
        }
    }
    CriticOutcome {
        findings,
        unparseable,
        usage,
        elapsed_ms: elapsed,
    }
}

fn parse_findings(text: &str) -> Option<Vec<String>> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    let parsed: Value = serde_json::from_str(&text[start..=end]).ok()?;
    let list = parsed["findings"].as_array()?;
    Some(
        list.iter()
            .filter_map(|finding| {
                let code = finding["code"].as_str()?;
                let quote = finding["quote"].as_str().unwrap_or_default();
                let reason = finding["reason"].as_str().unwrap_or_default();
                Some(format!("- [{code}] \"{quote}\" {reason}"))
            })
            .collect(),
    )
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
        Value::String(text) => {
            licensed.extend(validate::numeric_tokens(text));
        }
        _ => {}
    }
}

fn settings(max_tokens: u32) -> Value {
    json!({"model": MODEL, "max_tokens": max_tokens, "thinking": "disabled", "stream": false,
        "request_timeout_seconds": 60, "retries": 0})
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
    editor_prompt: &str,
    critic_prompt: &str,
    max_tokens: u32,
) -> Result<String> {
    let input = canonical(&json!({"fingerprint_version": FINGERPRINT_VERSION,
        "request": request, "system_prompt": prompt,
        "editor_prompt": editor_prompt, "critic_prompt": critic_prompt,
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

fn row(
    hostname: &str,
    status: &str,
    reason: &str,
    outcome: Option<&Outcome>,
) -> Value {
    let response = outcome.and_then(|o| o.response.as_ref());
    json!({"hostname": hostname, "status": status, "reason": reason,
        "model": response.and_then(|r|r["model"].as_str()),
        "finish_reason": response.and_then(|r|r["choices"][0]["finish_reason"].as_str()),
        "prompt_tokens": response.and_then(|r|r["usage"]["prompt_tokens"].as_u64()),
        "completion_tokens": response.and_then(|r|r["usage"]["completion_tokens"].as_u64()),
        "total_tokens": response.and_then(|r|r["usage"]["total_tokens"].as_u64()),
        "elapsed_ms": outcome.map(|o|o.elapsed_ms)})
}

fn preflight(
    options: &Options,
    prompt: &str,
    editor_prompt: &str,
    critic_prompt: &str,
) -> Result<(VecDeque<Job>, Vec<Value>)> {
    if options.concurrency == 0 || options.max_tokens == 0 {
        bail!("Concurrency and max_tokens must be positive");
    }
    if prompt.trim().is_empty() {
        bail!("System prompt must not be blank");
    }
    let filters: BTreeSet<_> =
        options.hostnames.iter().map(String::as_str).collect();
    let mut selected = BTreeMap::new();
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
        let request = read_toml(&request_path)?;
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
        let hash = fingerprint(
            &request,
            prompt,
            editor_prompt,
            critic_prompt,
            options.max_tokens,
        )?;
        let source_date =
            request["source"]["source_date"].as_str().map(str::to_owned);
        // Send deterministic TOML so key order/comments do not change the wire input
        // while the semantic fingerprint remains unchanged.
        let input = toml::to_string_pretty(&canonical(&request))?;
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
        let validation = validate::ValidationContext {
            planet_names,
            spectral_label,
            licensed_tokens,
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
                },
            )
            .is_some()
        {
            bail!("Duplicate prepared hostname");
        }
    }
    for name in &filters {
        if !selected.contains_key(*name) {
            bail!("No prepared request for {name}");
        }
    }
    if selected.is_empty() {
        bail!("No prepared requests found");
    }
    let mut jobs = VecDeque::new();
    let mut rows = Vec::new();
    for (_, job) in selected {
        let metadata_path = job.directory.join("metadata.toml");
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
        let description_path = job.directory.join("description.md");
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
        if !options.force && matches && !description.trim().is_empty() {
            rows.push(row(
                &job.hostname,
                "skipped",
                "generation inputs unchanged",
                None,
            ));
        } else {
            // Verify that per-system staging is writable before starting paid work.
            let staging = job.directory.join(".generate");
            fs::create_dir(&staging).with_context(|| {
                format!("Cannot write generation files for {}", job.hostname)
            })?;
            fs::remove_dir(&staging)?;
            for name in ["metadata.toml", "description.md", "fail.toml"] {
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
        draft_attempts,
        edit_attempts,
        draft_usage,
        edit_usage,
        critic_findings,
        critic_corrected,
        critic_unparseable,
        critic_usage,
        recovered,
        ..
    } = result;
    let mut doc = json!({"hostname": job.hostname, "fingerprint": job.fingerprint,
        "fingerprint_version": FINGERPRINT_VERSION, "settings": settings(max_tokens), "elapsed_ms": outcome.elapsed_ms,
        "attempts": attempts,
        "stages": {"draft": {"attempts": draft_attempts,
            "prompt_tokens": draft_usage.0, "completion_tokens": draft_usage.1},
            "edit": {"attempts": edit_attempts,
            "prompt_tokens": edit_usage.0, "completion_tokens": edit_usage.1}}});
    if !critic_findings.is_empty() || *critic_corrected || *critic_unparseable {
        doc["critic"] = json!({"findings": critic_findings,
            "corrected": critic_corrected, "unparseable": critic_unparseable,
            "prompt_tokens": critic_usage.0,
            "completion_tokens": critic_usage.1});
    }
    if !recovered.is_empty() {
        doc["validation_recovered"] =
            json!(recovered.iter().flatten().cloned().collect::<Vec<_>>());
    }
    if let Some(status) = outcome.status {
        doc["http_status"] = json!(status);
    }
    let report = row(&job.hostname, "", "", Some(outcome));
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
    fs::rename(&staged, job.directory.join("fail.toml"))?;
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
        [("description.md", text), ("metadata.toml", &metadata)],
        || Ok(()),
        |from, to| fs::rename(from, to),
    )?;
    match fs::remove_file(job.directory.join("fail.toml")) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => {
            return Err(e).context(
                "Result saved but previous fail.toml could not be removed",
            );
        }
    }
    Ok(())
}

async fn execute(
    mut jobs: VecDeque<Job>,
    mut rows: Vec<Value>,
    client: Client,
    prompt: String,
    editor_prompt: String,
    critic_prompt: String,
    concurrency: usize,
    max_tokens: u32,
) -> Result<Vec<Value>> {
    let prompt = Arc::new(prompt);
    let editor_prompt = Arc::new(editor_prompt);
    let critic_prompt = Arc::new(critic_prompt);
    let mut tasks = tokio::task::JoinSet::new();
    let mut halted = false;
    while !jobs.is_empty() || !tasks.is_empty() {
        while !halted && tasks.len() < concurrency {
            let Some(job) = jobs.pop_front() else {
                break;
            };
            let client = client.clone();
            let prompt = prompt.clone();
            let editor_prompt = editor_prompt.clone();
            let critic_prompt = critic_prompt.clone();
            eprintln!("Generating {}", job.hostname);
            tasks.spawn(async move {
                let result = generate_two_stage(
                    &client,
                    &job,
                    &prompt,
                    &editor_prompt,
                    &critic_prompt,
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
            draft_attempts,
            edit_attempts,
            failed_stage,
            final_errors,
            ..
        } = &result;
        let attempts = *attempts;
        let final_errors = final_errors.clone();
        let failed_stage = *failed_stage;
        let stage_attempts = if failed_stage == Some("editor") {
            *edit_attempts
        } else {
            *draft_attempts
        };
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
                    "{stage} validation failed after {stage_attempts} \
                     attempts: {}",
                    errors.join("; ")
                );
            }
        }
        if status == "failed" {
            if let Err(error) = save_failure(&job, max_tokens, &result, &reason) {
                halted = true;
                reason.push_str(&format!("; cannot save fail.toml: {error:#}"));
            }
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
        rows.push(row(&job.hostname, status, &reason, Some(&outcome)));
    }
    rows.extend(jobs.iter().map(|job| {
        row(
            &job.hostname,
            "not_started",
            "batch stopped after an account, rate-limit, or storage failure",
            None,
        )
    }));
    rows.sort_by(|a, b| a["hostname"].as_str().cmp(&b["hostname"].as_str()));
    Ok(rows)
}

pub fn run(options: &Options) -> Result<Vec<Value>> {
    let started = Instant::now();
    let prompt = fs::read_to_string(&options.system_prompt)?;
    let editor_prompt = fs::read_to_string(&options.editor_prompt)?;
    let critic_prompt = fs::read_to_string(&options.critic_prompt)?;
    let (jobs, mut rows) =
        preflight(options, &prompt, &editor_prompt, &critic_prompt)?;
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
                editor_prompt,
                critic_prompt,
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
