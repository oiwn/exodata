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
            concurrency: 4,
            max_tokens: 1536,
            force: false,
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
    let input: toml::Value = toml::from_str(
        body["messages"].as_array().unwrap().last().unwrap()["content"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let name = input["system"]["hostname"].as_str().unwrap();
    let action = input["test_action"].as_str().unwrap();
    tokio::time::sleep(Duration::from_millis(if name == "A" { 100 } else { 40 }))
        .await;
    state.active.fetch_sub(1, Ordering::SeqCst);
    if action == "storage" {
        fs::create_dir(state.directory.join(name).join(".generate")).unwrap();
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
    let content = if action == "empty" {
        json!("")
    } else {
        json!(format!("# {name}\nGenerated prose."))
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

#[test]
fn fingerprint_ignores_object_order_and_formatting_but_preserves_meaning() {
    let a: Value =
        serde_json::from_str(r#"{"b":{"y":2,"x":1},"a":[1,2]}"#).unwrap();
    let b: Value =
        serde_json::from_str("{\n \"a\": [1,2], \"b\": {\"x\":1,\"y\":2}} ")
            .unwrap();
    let hash = fingerprint(&a, "prompt", 1536).unwrap();
    assert_eq!(hash, fingerprint(&b, "prompt", 1536).unwrap());
    let mut changed = b.clone();
    changed["a"] = json!([2, 1]);
    assert_ne!(hash, fingerprint(&changed, "prompt", 1536).unwrap());
    changed = b;
    changed["b"]["x"] = json!(3);
    assert_ne!(hash, fingerprint(&changed, "prompt", 1536).unwrap());
    assert_ne!(hash, fingerprint(&a, "prompt ", 1536).unwrap());
    assert_ne!(hash, fingerprint(&a, "prompt", 1000).unwrap());
    assert_ne!(
        fingerprint(&json!({"text":"a b"}), "p", 1).unwrap(),
        fingerprint(&json!({"text":"a  b"}), "p", 1).unwrap()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrency_capture_skip_and_force() {
    let fixture = Fixture::new(&["A", "B", "C", "D", "E"]);
    let options = fixture.options();
    let prompt = fs::read_to_string(&options.system_prompt).unwrap();
    let (client, state, task) = server(&fixture, Duration::from_secs(5)).await;
    let (jobs, rows) = preflight(&options, &prompt).unwrap();
    let rows = execute(jobs, rows, client, prompt.clone(), 4, 1536)
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
        assert_eq!(body["max_tokens"], 1536);
        assert_eq!(body["thinking"]["type"], "disabled");
        assert_eq!(body["stream"], false);
        assert_eq!(body["messages"][0]["content"], prompt);
    }
    let (jobs, rows) = preflight(&options, &prompt).unwrap();
    assert!(jobs.is_empty());
    assert_eq!(rows.len(), 5);
    assert!(rows.iter().all(|r| r["status"] == "skipped"));
    let mut forced = options;
    forced.force = true;
    assert_eq!(preflight(&forced, &prompt).unwrap().0.len(), 5);
    assert_eq!(
        preflight(&fixture.options(), "changed prompt")
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
    let (jobs, rows) = preflight(&options, prompt).unwrap();
    execute(jobs, rows, client.clone(), prompt.into(), 1, 1536)
        .await
        .unwrap();
    let old = fs::read(fixture.0.join("A/description.md")).unwrap();
    let metadata = fs::read(fixture.0.join("A/metadata.toml")).unwrap();
    for action in ["length", "empty", "invalid", "http500"] {
        fixture.request("A", action);
        let (jobs, rows) = preflight(&options, prompt).unwrap();
        let rows = execute(jobs, rows, client.clone(), prompt.into(), 1, 1536)
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
        if action == "invalid" {
            assert_eq!(failure["response_body"], "not JSON");
        }
        if action == "http500" {
            assert!(failure.get("response_body").is_none());
        }
        assert_eq!(preflight(&options, prompt).unwrap().0.len(), 1);
    }
    fixture.request("A", "ok");
    let mut force = fixture.options();
    force.force = true;
    let (jobs, rows) = preflight(&force, prompt).unwrap();
    execute(jobs, rows, client, prompt.into(), 1, 1536)
        .await
        .unwrap();
    assert!(!fixture.0.join("A/fail.toml").exists());
    assert_eq!(state.calls.load(Ordering::SeqCst), 6);
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
        let (jobs, rows) = preflight(&fixture.options(), "prompt").unwrap();
        let rows = execute(jobs, rows, client, "prompt".into(), 1, 1536)
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
async fn timeout_is_recorded_without_retry() {
    let fixture = Fixture::new(&["A"]);
    let (client, state, task) = server(&fixture, Duration::from_millis(20)).await;
    let (jobs, rows) = preflight(&fixture.options(), "prompt").unwrap();
    let rows = execute(jobs, rows, client, "prompt".into(), 1, 1536)
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
    let (jobs, rows) = preflight(&fixture.options(), "prompt").unwrap();
    let rows = execute(jobs, rows, client, "prompt".into(), 2, 1536)
        .await
        .unwrap();
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
    assert_eq!(preflight(&options, "prompt").unwrap().0[0].hostname, "B");
    options.hostnames = vec!["missing".into()];
    assert!(preflight(&options, "prompt").is_err());
    options.hostnames.clear();
    options.concurrency = 0;
    assert!(preflight(&options, "prompt").is_err());
    options.concurrency = 4;
    assert!(preflight(&options, " ").is_err());
    fs::remove_file(fixture.0.join("B/evidence.json")).unwrap();
    assert!(preflight(&options, "prompt").is_err());
    fixture.request("B", "ok");
    fs::write(fixture.0.join("B/evidence.json"), r#"{"hostname":"A"}"#).unwrap();
    assert!(preflight(&options, "prompt").is_err());
}
