//! Native generation of saved requests, with current per-system results only.
use super::client::{Client, MODEL, Outcome};
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

const FINGERPRINT_VERSION: u32 = 1;

#[cfg(test)]
mod tests;

pub struct Options {
    pub input_dir: PathBuf,
    pub hostnames: Vec<String>,
    pub system_prompt: PathBuf,
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

fn fingerprint(request: &Value, prompt: &str, max_tokens: u32) -> Result<String> {
    let input = canonical(&json!({"fingerprint_version": FINGERPRINT_VERSION,
        "request": request, "system_prompt": prompt, "settings": settings(max_tokens)}));
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
        let hash = fingerprint(&request, prompt, options.max_tokens)?;
        let source_date =
            request["source"]["source_date"].as_str().map(str::to_owned);
        // Send deterministic TOML so key order/comments do not change the wire input
        // while the semantic fingerprint remains unchanged.
        let input = toml::to_string_pretty(&canonical(&request))?;
        if selected
            .insert(
                hostname.clone(),
                Job {
                    directory,
                    hostname,
                    input,
                    fingerprint: hash,
                    source_date,
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

fn record(job: &Job, max_tokens: u32, outcome: &Outcome) -> Value {
    let mut doc = json!({"hostname": job.hostname, "fingerprint": job.fingerprint,
        "fingerprint_version": FINGERPRINT_VERSION, "settings": settings(max_tokens), "elapsed_ms": outcome.elapsed_ms});
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
    outcome: &Outcome,
    reason: &str,
) -> Result<()> {
    let mut doc = record(job, max_tokens, outcome);
    doc["failed_at"] = json!(now());
    doc["error"] = json!(reason);
    if let Some(body) = &outcome.body {
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

fn save_success(job: &Job, max_tokens: u32, outcome: &Outcome) -> Result<()> {
    let report = outcome
        .report
        .as_ref()
        .context("Missing successful response")?;
    let text = report["text"].as_str().context("Missing description")?;
    let mut doc = record(job, max_tokens, outcome);
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
    concurrency: usize,
    max_tokens: u32,
) -> Result<Vec<Value>> {
    let prompt = Arc::new(prompt);
    let mut tasks = tokio::task::JoinSet::new();
    let mut halted = false;
    while !jobs.is_empty() || !tasks.is_empty() {
        while !halted && tasks.len() < concurrency {
            let Some(job) = jobs.pop_front() else {
                break;
            };
            let client = client.clone();
            let prompt = prompt.clone();
            eprintln!("Generating {}", job.hostname);
            tasks.spawn(async move {
                let outcome =
                    client.generate(&job.input, Some(&prompt), max_tokens).await;
                (job, outcome)
            });
        }
        let Some(finished) = tasks.join_next().await else {
            break;
        };
        let (job, outcome) = finished.context("Generation worker failed")?;
        halted |= outcome.stop_batch();
        let mut reason = outcome.error.clone().unwrap_or_default();
        let mut status = "generated";
        if outcome.report.is_some() {
            if let Err(error) = save_success(&job, max_tokens, &outcome) {
                halted = true;
                reason = format!("Storage failure: {error:#}");
                status = "failed";
            }
        } else {
            status = "failed";
        }
        if status == "failed" {
            if let Err(error) = save_failure(&job, max_tokens, &outcome, &reason)
            {
                halted = true;
                reason.push_str(&format!("; cannot save fail.toml: {error:#}"));
            }
        }
        eprintln!(
            "{}: {status}{}",
            job.hostname,
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
    let (jobs, mut rows) = preflight(options, &prompt)?;
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
