use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::time::{Duration, Instant};

pub(super) const MODEL: &str = "deepseek-v4-flash";
const ENDPOINT: &str = "https://api.deepseek.com/chat/completions";

#[derive(Clone)]
pub(super) struct Client {
    http: reqwest::Client,
    key: String,
    endpoint: String,
}

pub(super) struct Outcome {
    pub status: Option<u16>,
    pub elapsed_ms: u128,
    pub body: Option<String>,
    pub response: Option<Value>,
    pub report: Option<Value>,
    pub error: Option<String>,
}

impl Outcome {
    pub fn stop_batch(&self) -> bool {
        matches!(self.status, Some(401 | 402 | 403 | 429))
    }
}

impl Client {
    pub fn from_env() -> Result<Self> {
        let key = std::env::var("DEEPSEEK_API_KEY").context(
            "Set DEEPSEEK_API_KEY in the environment before calling DeepSeek",
        )?;
        Self::new(key.trim(), ENDPOINT, Duration::from_secs(60))
    }

    pub(super) fn new(
        key: &str,
        endpoint: &str,
        timeout: Duration,
    ) -> Result<Self> {
        if key.trim().is_empty() {
            bail!("DEEPSEEK_API_KEY must not be blank");
        }
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(timeout)
                .redirect(reqwest::redirect::Policy::none())
                .retry(reqwest::retry::never())
                .build()?,
            key: key.to_owned(),
            endpoint: endpoint.to_owned(),
        })
    }

    pub async fn generate(
        &self,
        input: &str,
        prompt: Option<&str>,
        max_tokens: u32,
    ) -> Outcome {
        let started = Instant::now();
        let mut outcome = Outcome {
            status: None,
            elapsed_ms: 0,
            body: None,
            response: None,
            report: None,
            error: None,
        };
        let response = self
            .http
            .post(&self.endpoint)
            .bearer_auth(&self.key)
            .json(&super::probe::request_body(input, prompt, max_tokens))
            .send()
            .await;
        match response {
            Err(error) => {
                outcome.error = Some(if error.is_timeout() {
                    "DeepSeek request timed out; no retry attempted".into()
                } else {
                    "DeepSeek connection/request failed; no retry attempted"
                        .into()
                })
            }
            Ok(response) => {
                let status = response.status();
                outcome.status = Some(status.as_u16());
                if !status.is_success() {
                    let reason = match status.as_u16() {
                        401 | 403 => "authentication/authorization failed",
                        402 => "insufficient balance",
                        429 => "rate limited",
                        _ => "request failed",
                    };
                    outcome.error = Some(format!(
                        "DeepSeek HTTP {status}: {reason}; no retry attempted"
                    ));
                } else {
                    match response.text().await {
                        Err(_) => outcome.error = Some("DeepSeek response body could not be read; no retry attempted".into()),
                        Ok(body) => {
                            // Never persist a credential even if a remote response unexpectedly echoes it.
                            let body = body.replace(&self.key, "[REDACTED]");
                            outcome.response = serde_json::from_str(&body).ok();
                            let report = serde_json::from_str::<super::probe::Response>(&body)
                                .context("DeepSeek response could not be read or parsed")
                                .and_then(|r| super::probe::report(r, started.elapsed().as_millis()));
                            match report {
                                Ok(report) => outcome.report = Some(report),
                                Err(error) => outcome.error = Some(error.to_string()),
                            }
                            outcome.body = Some(body);
                        }
                    }
                }
            }
        }
        outcome.elapsed_ms = started.elapsed().as_millis();
        outcome
    }
}
