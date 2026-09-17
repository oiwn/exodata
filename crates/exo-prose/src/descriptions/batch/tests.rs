use super::*;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(names: &[&str]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "exodata-batch-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("prompt.txt"), "Use supplied facts.").unwrap();
        fs::write(
            root.join("style-prompt.txt"),
            "Polish the draft; preserve facts.",
        )
        .unwrap();
        let fixture = Self(root);
        for name in names {
            fixture.request(name, "ok");
        }
        fixture
    }
    fn request(&self, name: &str, action: &str) {
        let dir = self.0.join(name);
        fs::create_dir_all(&dir).unwrap();
        let request = json!({"schema_version":1,"system":{"hostname":name},"planets":[],"test_action":action});
        fs::write(dir.join("request.toml"), toml::to_string(&request).unwrap())
            .unwrap();
        fs::write(
            dir.join("evidence.json"),
            serde_json::to_string(&json!({"hostname":name})).unwrap(),
        )
        .unwrap();
    }
    fn options(&self) -> Options {
        Options {
            input_dir: self.0.clone(),
            hostnames: vec![],
            system_prompt: self.0.join("prompt.txt"),
            repair_prompt: self.0.join("style-prompt.txt"),
            concurrency: 4,
            max_tokens: 1536,
            force: false,
            failed: false,
            label: None,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Default)]
struct ServerState {
    active: AtomicUsize,
    peak: AtomicUsize,
    calls: AtomicUsize,
    requests: std::sync::Mutex<Vec<Value>>,
    directory: PathBuf,
}

async fn handle(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<Value>,
) -> Response {
    state.calls.fetch_add(1, Ordering::SeqCst);
    let active = state.active.fetch_add(1, Ordering::SeqCst) + 1;
    state.peak.fetch_max(active, Ordering::SeqCst);
    state.requests.lock().unwrap().push(body.clone());
    let content_str =
        body["messages"].as_array().unwrap().last().unwrap()["content"]
            .as_str()
            .unwrap()
            .to_owned();
    let wire: Value = serde_json::from_str(&content_str).unwrap();
    let repair_stage = wire.get("facts").is_some();
    let input = if repair_stage { &wire["facts"] } else { &wire };
    let name = input["system"]["hostname"].as_str().unwrap().to_owned();
    // Test control stays in the fixture; it must never leak into wire facts.
    let action = read_toml(&state.directory.join(&name).join("request.toml"))
        .unwrap()["test_action"]
        .as_str()
        .unwrap()
        .to_owned();
    tokio::time::sleep(Duration::from_millis(if name == "A" { 100 } else { 40 }))
        .await;
    state.active.fetch_sub(1, Ordering::SeqCst);
    if action == "storage" {
        let _ = fs::create_dir(state.directory.join(&name).join(".generate"));
    }
    if let Some(status) = action
        .strip_prefix("http")
        .and_then(|s| s.parse::<u16>().ok())
    {
        return (
            StatusCode::from_u16(status).unwrap(),
            "private remote diagnostic",
        )
            .into_response();
    }
    if action == "invalid" {
        return (StatusCode::OK, "not JSON").into_response();
    }
    if action == "repair-http500" && repair_stage {
        return (StatusCode::INTERNAL_SERVER_ERROR, "repair unavailable")
            .into_response();
    }
    let content = match action.as_str() {
        "retry" if repair_stage => {
            json!(format!("# {name}\n**Planet b** orbits **about 3 days**."))
        }
        "retry" | "unlicensed" => {
            json!(format!("# {name}\n**Planet b** orbits **about 7 days**."))
        }
        "stylefail" | "repair-http500" => {
            json!(format!("# {name}\nIt orbits in 42 days."))
        }
        "assoc" => json!(format!(
            "# {name}\nOne planet is associated with this host."
        )),
        "emdash" => json!(format!("# {name}\nOne — two.")),
        "machine" => json!(format!(
            "# {name}\nIts mass is large. Its radius is small. Its year is short. Its density is high."
        )),
        "empty" => json!(""),
        _ => json!(format!("# {name}\nGenerated prose.")),
    };
    Json(json!({"model":MODEL,"id":"mock-response-id",
        "choices":[{"message":{"content":content},"finish_reason":if action=="length"{"length"}else{"stop"}}],
        "usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}})).into_response()
}

async fn server(
    fixture: &Fixture,
    timeout: Duration,
) -> (Client, Arc<ServerState>, tokio::task::JoinHandle<()>) {
    let state = Arc::new(ServerState {
        directory: fixture.0.clone(),
        ..Default::default()
    });
    let app = Router::new()
        .route("/", post(handle))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (
        Client::new("mock-secret-credential", &endpoint, timeout).unwrap(),
        state,
        task,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_filter_retries_failed_systems_over_matching_fingerprints() {
    let fixture = Fixture::new(&["A", "B"]);
    let options = fixture.options();
    let prompt = "prompt";
    let (client, _state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
        .await
        .unwrap();
    // Simulate a failed forced rerun with unchanged inputs: the success
    // metadata still matches the current fingerprint, but a fail.toml
    // records the latest, failed attempt.
    fs::write(
        fixture.0.join("A/fail.toml"),
        "hostname = 'A'\nerror = 'forced rerun failed'\n",
    )
    .unwrap();
    let plain = fixture.options();
    assert!(preflight(&plain, prompt, "style").unwrap().0.is_empty());
    let mut retry = fixture.options();
    retry.failed = true;
    let (jobs, _rows) = preflight(&retry, prompt, "style").unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].hostname, "A");
    let mut missing = retry.clone();
    missing.hostnames = vec!["B".into()];
    assert!(preflight(&missing, prompt, "style").is_err());
    task.abort();
}

#[test]
fn notes_merge_into_wire_input_and_change_the_fingerprint() {
    let fixture = Fixture::new(&["A"]);
    let options = fixture.options();
    let (jobs, _) = preflight(&options, "prompt", "style").unwrap();
    let plain = jobs[0].fingerprint.clone();
    let input = jobs[0].input.clone();
    fs::write(
        fixture.0.join("A/notes.toml"),
        "facts = ['The planet glows faintly in X-ray.']\n\
         guidance = ['Mention the X-ray fact last.']\n",
    )
    .unwrap();
    let (jobs, _) = preflight(&options, "prompt", "style").unwrap();
    assert_ne!(jobs[0].fingerprint, plain);
    assert!(jobs[0].input.contains("The planet glows faintly in X-ray."));
    assert!(jobs[0].input.contains("Mention the X-ray fact last."));
    assert!(!input.contains("X-ray"));
}

#[test]
fn young_age_alias_requires_an_explicit_request_license() {
    let title = "# A Young Red Dwarf and Its Single Imaged Planet\n\nA star.";
    let request = json!({"star": {"age_class": "very young"}});
    let licensed = licensed_phrases(&request, &None);
    assert!(validate::banned_patterns(title, &licensed).is_ok());
    let age_only = json!({"star": {"measurements": {"age": {"display": "about 0.005 billion years"}}}});
    assert!(
        validate::banned_patterns(title, &licensed_phrases(&age_only, &None))
            .is_err()
    );
    assert!(
        validate::banned_patterns("# The youngest star\n\nA star.", &licensed)
            .is_err()
    );
}

#[test]
fn fingerprint_ignores_object_order_and_formatting_but_preserves_meaning() {
    let a: Value =
        serde_json::from_str(r#"{"b":{"y":2,"x":1},"a":[1,2]}"#).unwrap();
    let b: Value =
        serde_json::from_str("{\n \"a\": [1,2], \"b\": {\"x\":1,\"y\":2}} ")
            .unwrap();
    let hash = fingerprint(&a, "prompt", "style", 1536).unwrap();
    assert_eq!(hash, fingerprint(&b, "prompt", "style", 1536).unwrap());
    let mut changed = b.clone();
    changed["a"] = json!([2, 1]);
    assert_ne!(
        hash,
        fingerprint(&changed, "prompt", "style", 1536).unwrap()
    );
    changed = b;
    changed["b"]["x"] = json!(3);
    assert_ne!(
        hash,
        fingerprint(&changed, "prompt", "style", 1536).unwrap()
    );
    assert_ne!(hash, fingerprint(&a, "prompt ", "style", 1536).unwrap());
    assert_ne!(hash, fingerprint(&a, "prompt", "style", 1000).unwrap());
    assert_ne!(
        fingerprint(&json!({"text":"a b"}), "p", "style", 1).unwrap(),
        fingerprint(&json!({"text":"a  b"}), "p", "style", 1).unwrap()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrency_capture_skip_and_force() {
    let fixture = Fixture::new(&["A", "B", "C", "D", "E"]);
    let options = fixture.options();
    let prompt = fs::read_to_string(&options.system_prompt).unwrap();
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, &prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.clone(), "style".into(), 4, 1536)
            .await
            .unwrap();
    assert_eq!(state.peak.load(Ordering::SeqCst), 4);
    assert_eq!(state.calls.load(Ordering::SeqCst), 5);
    assert!(rows.iter().all(|r| r["status"] == "generated"));
    assert_eq!(rows[0]["hostname"], "A");
    for name in ["A", "B", "C", "D", "E"] {
        let directory = fixture.0.join(name);
        let metadata = read_toml(&directory.join("metadata.toml")).unwrap();
        assert_eq!(metadata["total_tokens"], 15);
        assert_eq!(metadata["settings"]["max_tokens"], 1536);
        assert!(
            fs::read_to_string(directory.join("description.md"))
                .unwrap()
                .contains(name)
        );
        assert!(!directory.join("fail.toml").exists());
        assert!(!directory.join(".generate").exists());
    }
    for body in state.requests.lock().unwrap().iter() {
        assert_eq!(body["model"], MODEL);
        let system = body["messages"][0]["content"].as_str().unwrap();
        assert_eq!(body["max_tokens"], 1536);
        assert_eq!(body["thinking"]["type"], "disabled");
        assert_eq!(body["stream"], false);
        assert!(body.get("response_format").is_none());
        assert!(system == prompt || system == "style");
    }
    let (jobs, rows) = preflight(&options, &prompt, "style").unwrap();
    assert!(jobs.is_empty());
    assert_eq!(rows.len(), 5);
    assert!(rows.iter().all(|r| r["status"] == "skipped"));
    let mut forced = options;
    forced.force = true;
    assert_eq!(preflight(&forced, &prompt, "style").unwrap().0.len(), 5);
    assert_eq!(
        preflight(&fixture.options(), "changed prompt", "style")
            .unwrap()
            .0
            .len(),
        5
    );
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn failure_preserves_previous_success_and_later_success_clears_failure() {
    let fixture = Fixture::new(&["A"]);
    let options = fixture.options();
    let prompt = "prompt";
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    execute(
        jobs,
        rows,
        client.clone(),
        prompt.into(),
        "style".into(),
        1,
        1536,
    )
    .await
    .unwrap();
    let old = fs::read(fixture.0.join("A/description.md")).unwrap();
    let metadata = fs::read(fixture.0.join("A/metadata.toml")).unwrap();
    for action in ["length", "empty", "invalid", "http500"] {
        fixture.request("A", action);
        let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
        let rows = execute(
            jobs,
            rows,
            client.clone(),
            prompt.into(),
            "style".into(),
            1,
            1536,
        )
        .await
        .unwrap();
        assert_eq!(rows[0]["status"], "failed");
        assert_eq!(fs::read(fixture.0.join("A/description.md")).unwrap(), old);
        assert_eq!(
            fs::read(fixture.0.join("A/metadata.toml")).unwrap(),
            metadata
        );
        let failure = read_toml(&fixture.0.join("A/fail.toml")).unwrap();
        assert!(failure["error"].as_str().is_some());
        assert!(
            !serde_json::to_string(&failure)
                .unwrap()
                .contains("mock-secret-credential")
        );
        if action == "length" {
            assert_eq!(failure["finish_reason"], "length");
            assert_eq!(failure["total_tokens"], 15);
            assert!(
                failure["response_body"]
                    .as_str()
                    .unwrap()
                    .contains("Generated prose")
            );
        }
        if action == "empty" {
            assert!(failure["error"].as_str().unwrap().contains("empty text"));
        }
        if action == "invalid" {
            assert_eq!(failure["response_body"], "not JSON");
        }
        if action == "http500" {
            assert!(failure.get("response_body").is_none());
        }
        assert_eq!(preflight(&options, prompt, "style").unwrap().0.len(), 1);
    }
    fixture.request("A", "ok");
    let mut force = fixture.options();
    force.force = true;
    let (jobs, rows) = preflight(&force, prompt, "style").unwrap();
    execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
        .await
        .unwrap();
    assert!(!fixture.0.join("A/fail.toml").exists());
    assert_eq!(state.calls.load(Ordering::SeqCst), 6);
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn validation_retry_recovers_and_records_attempts() {
    let fixture = Fixture::new(&[]);
    let dir = fixture.0.join("Retry");
    fs::create_dir_all(&dir).unwrap();
    let request = json!({"schema_version":1,"system":{"hostname":"Retry"},
        "planets":[{"name":"Planet b"}],
        "publishable_comparisons":["Planet b orbits in about 3 days."],
        "test_action":"retry"});
    fs::write(dir.join("request.toml"), toml::to_string(&request).unwrap())
        .unwrap();
    fs::write(
        dir.join("evidence.json"),
        serde_json::to_string(&json!({"hostname":"Retry"})).unwrap(),
    )
    .unwrap();
    let options = fixture.options();
    let prompt = "prompt";
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "generated");
    // One draft and one targeted repair, which may correct the invalid number.
    assert_eq!(state.calls.load(Ordering::SeqCst), 2);
    let description = fs::read_to_string(dir.join("description.md")).unwrap();
    assert!(description.contains("**Planet b** orbits **3 days**."));
    let draft = fs::read_to_string(dir.join("draft.md")).unwrap();
    assert!(draft.contains("**Planet b** orbits **7 days**."));
    let metadata = read_toml(&dir.join("metadata.toml")).unwrap();
    assert_eq!(metadata["attempts"], 2);
    assert_eq!(metadata["stages"]["draft"]["attempts"], 1);
    assert_eq!(metadata["stages"]["repair"]["attempts"], 1);
    assert!(
        metadata["validation_recovered"]
            .as_array()
            .is_some_and(|errors| !errors.is_empty())
    );
    assert_eq!(metadata["total_tokens"], 30);
    assert_eq!(rows[0]["total_tokens"], 30);
    assert_eq!(metadata["usage_complete"], true);
    let requests = state.requests.lock().unwrap();
    let repair: Value = serde_json::from_str(
        requests[1]["messages"][1]["content"].as_str().unwrap(),
    )
    .unwrap();
    assert!(repair["article"].as_str().unwrap().contains("7 days"));
    assert!(
        repair["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("not licensed"))
    );
    assert_eq!(repair["facts"]["system"]["hostname"], "Retry");
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn labeled_runs_write_variant_files_and_keep_served_output() {
    let fixture = Fixture::new(&["A"]);
    let options = fixture.options();
    let prompt = "prompt";
    let (client, _state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
        .await
        .unwrap();
    let served = fs::read_to_string(fixture.0.join("A/description.md")).unwrap();
    let mut variant = options;
    variant.label = Some("v2".into());
    variant.force = true;
    let (client, _state, task2) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&variant, prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "generated");
    let written =
        fs::read_to_string(fixture.0.join("A/description_v2.md")).unwrap();
    assert!(written.contains("Generated prose."));
    assert!(fixture.0.join("A/metadata_v2.toml").exists());
    assert!(fixture.0.join("A/draft_v2.md").exists());
    assert_eq!(
        fs::read_to_string(fixture.0.join("A/description.md")).unwrap(),
        served,
        "the served description is untouched"
    );
    task.abort();
    task2.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn valid_draft_finishes_in_one_call_without_a_repair() {
    let fixture = Fixture::new(&["A"]);
    let options = fixture.options();
    let prompt = "prompt";
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "generated");
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    let description =
        fs::read_to_string(fixture.0.join("A/description.md")).unwrap();
    assert!(description.contains("Generated prose."));
    let metadata = read_toml(&fixture.0.join("A/metadata.toml")).unwrap();
    assert_eq!(metadata["stages"]["draft"]["attempts"], 1);
    assert!(metadata["stages"].get("repair").is_none());
    assert_eq!(metadata["total_tokens"], 15);
    assert!(metadata.get("critic").is_none());
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn labeled_failed_retry_preserves_other_artifacts_and_clears_only_its_failure()
 {
    let fixture = Fixture::new(&["A", "B"]);
    let mut options = fixture.options();
    options.label = Some("v2".into());
    let (client, _state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, "prompt", "style").unwrap();
    execute(
        jobs,
        rows,
        client.clone(),
        "prompt".into(),
        "style".into(),
        1,
        1536,
    )
    .await
    .unwrap();
    let dir = fixture.0.join("A");
    fs::write(dir.join("description.md"), "served").unwrap();
    fs::write(dir.join("fail.toml"), "error = 'served failed'\n").unwrap();
    fs::write(dir.join("fail_other.toml"), "error = 'other failed'\n").unwrap();
    fs::write(dir.join("fail_v2.toml"), "error = 'forced retry failed'\n")
        .unwrap();
    assert!(preflight(&options, "prompt", "style").unwrap().0.is_empty());
    options.failed = true;
    let (jobs, rows) = preflight(&options, "prompt", "style").unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].hostname, "A");
    let rows =
        execute(jobs, rows, client, "prompt".into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "generated");
    assert!(!dir.join("fail_v2.toml").exists());
    assert!(dir.join("fail.toml").exists());
    assert!(dir.join("fail_other.toml").exists());
    assert_eq!(
        fs::read_to_string(dir.join("description.md")).unwrap(),
        "served"
    );

    // Matching hashes with an old pipeline version must not skip generation.
    options.failed = false;
    options.hostnames = vec!["A".into()];
    let mut metadata = read_toml(&dir.join("metadata_v2.toml")).unwrap();
    assert_eq!(metadata["fingerprint_version"], 10);
    metadata["fingerprint_version"] = json!(5);
    fs::write(
        dir.join("metadata_v2.toml"),
        toml::to_string(&metadata).unwrap(),
    )
    .unwrap();
    assert_eq!(preflight(&options, "prompt", "style").unwrap().0.len(), 1);

    let labeled =
        super::super::analyze::Corpus::load(&fixture.0, Some("v2")).unwrap();
    assert!(!labeled.systems.iter().any(|doc| doc.failed));
    fs::write(dir.join("fail_v2.toml"), "error = 'latest failed'\n").unwrap();
    let labeled =
        super::super::analyze::Corpus::load(&fixture.0, Some("v2")).unwrap();
    assert!(labeled.systems[0].failed);
    assert!(labeled.systems[0].described);
    assert!(!labeled.systems[1].failed);
    assert!(
        super::super::analyze::Corpus::load(&fixture.0, Some("../bad")).is_err()
    );
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn repair_failure_counts_both_calls_and_preserves_previous() {
    let fixture = Fixture::new(&["A"]);
    let options = fixture.options();
    let prompt = "prompt";
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    execute(
        jobs,
        rows,
        client.clone(),
        prompt.into(),
        "style".into(),
        1,
        1536,
    )
    .await
    .unwrap();
    let old = fs::read(fixture.0.join("A/description.md")).unwrap();
    fixture.request("A", "stylefail");
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "failed");
    assert!(
        rows[0]["reason"]
            .as_str()
            .unwrap()
            .contains("repair validation failed")
    );
    // First run uses one call; the failed run uses draft plus one repair.
    assert_eq!(state.calls.load(Ordering::SeqCst), 3);
    assert_eq!(fs::read(fixture.0.join("A/description.md")).unwrap(), old);
    let failure = read_toml(&fixture.0.join("A/fail.toml")).unwrap();
    assert_eq!(failure["stages"]["repair"]["attempts"], 1);
    assert_eq!(failure["total_tokens"], 30);
    assert_eq!(rows[0]["total_tokens"], 30);
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn ordinary_repetition_does_not_trigger_repair_and_notes_reach_the_writer()
{
    let fixture = Fixture::new(&["A"]);
    fixture.request("A", "assoc");
    fs::write(
        fixture.0.join("A/notes.toml"),
        "facts = ['A has a companion.']\nguidance = ['Use compact prose.']\n",
    )
    .unwrap();
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&fixture.options(), "prompt", "repair").unwrap();
    let rows = execute(
        jobs,
        rows,
        client,
        "prompt".into(),
        "repair".into(),
        1,
        1536,
    )
    .await
    .unwrap();
    assert_eq!(rows[0]["status"], "generated");
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    let bodies = state.requests.lock().unwrap();
    let facts: Value = serde_json::from_str(
        bodies[0]["messages"][1]["content"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(facts["publishable_comparisons"][0], "A has a companion.");
    assert_eq!(facts["silent_constraints"][0], "Use compact prose.");
    assert!(
        fs::read_to_string(fixture.0.join("A/description.md"))
            .unwrap()
            .contains("associated with")
    );
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn em_dashes_are_fixed_mechanically_without_retries() {
    let fixture = Fixture::new(&["A"]);
    fixture.request("A", "emdash");
    let options = fixture.options();
    let prompt = "prompt";
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "generated");
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    let draft = fs::read_to_string(fixture.0.join("A/draft.md")).unwrap();
    assert!(draft.contains("One - two."), "dash was normalized in place");
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn uniform_writer_output_does_not_trigger_retries() {
    let fixture = Fixture::new(&["A"]);
    fixture.request("A", "machine");
    let options = fixture.options();
    let prompt = "prompt";
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "generated");
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn exhausted_validation_preserves_previous_description() {
    let fixture = Fixture::new(&["A"]);
    let options = fixture.options();
    let prompt = "prompt";
    let (client, _state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    execute(
        jobs,
        rows,
        client.clone(),
        prompt.into(),
        "style".into(),
        1,
        1536,
    )
    .await
    .unwrap();
    let old = fs::read(fixture.0.join("A/description.md")).unwrap();
    fixture.request("A", "unlicensed");
    let (jobs, rows) = preflight(&options, prompt, "style").unwrap();
    let rows =
        execute(jobs, rows, client, prompt.into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "failed");
    assert!(
        rows[0]["reason"]
            .as_str()
            .unwrap()
            .contains("repair validation failed after 2 total calls")
    );
    assert_eq!(fs::read(fixture.0.join("A/description.md")).unwrap(), old);
    let failure = read_toml(&fixture.0.join("A/fail.toml")).unwrap();
    assert_eq!(failure["attempts"], 2);
    assert!(failure["error"].as_str().unwrap().contains("not licensed"));
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn fatal_errors_stop_scheduling_but_server_errors_do_not() {
    for action in [
        "http401", "http402", "http403", "http429", "http500", "storage",
    ] {
        let fixture = Fixture::new(&["A", "B"]);
        fixture.request("A", action);
        let (client, state, task) =
            server(&fixture, Duration::from_secs(5)).await;
        let (jobs, rows) =
            preflight(&fixture.options(), "prompt", "style").unwrap();
        let rows =
            execute(jobs, rows, client, "prompt".into(), "style".into(), 1, 1536)
                .await
                .unwrap();
        let continues = action == "http500";
        assert_eq!(
            state.calls.load(Ordering::SeqCst),
            if continues { 2 } else { 1 }
        );
        assert_eq!(rows[0]["status"], "failed");
        assert_eq!(
            rows[1]["status"],
            if continues {
                "generated"
            } else {
                "not_started"
            }
        );
        assert!(fixture.0.join("A/fail.toml").exists());
        task.abort();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn repair_transport_failure_keeps_draft_usage_and_marks_it_incomplete() {
    let fixture = Fixture::new(&["A"]);
    fixture.request("A", "repair-http500");
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&fixture.options(), "prompt", "repair").unwrap();
    let rows = execute(
        jobs,
        rows,
        client,
        "prompt".into(),
        "repair".into(),
        1,
        1536,
    )
    .await
    .unwrap();
    assert_eq!(state.calls.load(Ordering::SeqCst), 2);
    assert_eq!(rows[0]["status"], "failed");
    assert_eq!(rows[0]["total_tokens"], 15);
    assert_eq!(rows[0]["usage_complete"], false);
    let failure = read_toml(&fixture.0.join("A/fail.toml")).unwrap();
    assert_eq!(failure["total_tokens"], 15);
    assert_eq!(failure["usage_complete"], false);
    assert!(failure["elapsed_ms"].as_u64().unwrap() >= 200);
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn timeout_is_recorded_without_retry() {
    let fixture = Fixture::new(&["A"]);
    let (client, state, task) = server(&fixture, Duration::from_millis(20)).await;
    let (jobs, rows) = preflight(&fixture.options(), "prompt", "style").unwrap();
    let rows =
        execute(jobs, rows, client, "prompt".into(), "style".into(), 1, 1536)
            .await
            .unwrap();
    assert_eq!(rows[0]["status"], "failed");
    assert!(
        read_toml(&fixture.0.join("A/fail.toml")).unwrap()["error"]
            .as_str()
            .unwrap()
            .contains("timed out")
    );
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    task.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn fatal_response_finishes_already_running_requests() {
    let fixture = Fixture::new(&["A", "B", "C"]);
    // B fails before the slower A finishes; C must never start.
    fixture.request("B", "http429");
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&fixture.options(), "prompt", "style").unwrap();
    let rows =
        execute(jobs, rows, client, "prompt".into(), "style".into(), 2, 1536)
            .await
            .unwrap();
    // A completes draft + style; B fails at its draft; C never starts.
    assert_eq!(state.calls.load(Ordering::SeqCst), 2);
    assert_eq!(rows[0]["status"], "generated");
    assert_eq!(rows[1]["status"], "failed");
    assert_eq!(rows[2]["status"], "not_started");
    assert!(fixture.0.join("A/description.md").exists());
    task.abort();
}

#[test]
fn preflight_filters_and_rejects_missing_or_mismatched_inputs() {
    let fixture = Fixture::new(&["A", "B"]);
    let mut options = fixture.options();
    options.hostnames = vec!["B".into()];
    assert_eq!(
        preflight(&options, "prompt", "style").unwrap().0[0].hostname,
        "B"
    );
    options.hostnames = vec!["missing".into()];
    assert!(preflight(&options, "prompt", "style").is_err());
    options.hostnames.clear();
    options.concurrency = 0;
    assert!(preflight(&options, "prompt", "style").is_err());
    options.concurrency = 4;
    assert!(preflight(&options, " ", "style").is_err());
    fs::remove_file(fixture.0.join("B/evidence.json")).unwrap();
    assert!(preflight(&options, "prompt", "style").is_err());
    fixture.request("B", "ok");
    fs::write(fixture.0.join("B/evidence.json"), r#"{"hostname":"A"}"#).unwrap();
    assert!(preflight(&options, "prompt", "style").is_err());
}
