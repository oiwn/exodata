//! Usage survives rejected prose and missing responses; unknown is not zero.
use serde_json::{Value, json};

#[derive(Clone, Copy, Default)]
pub(super) struct Usage {
    pub prompt: u64,
    pub completion: u64,
    pub total: u64,
    pub complete: bool,
}

impl Usage {
    pub fn read(value: &Value) -> Self {
        let prompt = value["prompt_tokens"].as_u64();
        let completion = value["completion_tokens"].as_u64();
        Self {
            prompt: prompt.unwrap_or_default(),
            completion: completion.unwrap_or_default(),
            total: value["total_tokens"].as_u64().unwrap_or(
                prompt.unwrap_or_default() + completion.unwrap_or_default(),
            ),
            complete: value["usage_complete"]
                .as_bool()
                .unwrap_or(prompt.is_some() && completion.is_some()),
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            prompt: self.prompt + other.prompt,
            completion: self.completion + other.completion,
            total: self.total + other.total,
            complete: self.complete && other.complete,
        }
    }

    pub fn write(self, value: &mut Value) {
        value["prompt_tokens"] = json!(self.prompt);
        value["completion_tokens"] = json!(self.completion);
        value["total_tokens"] = json!(self.total);
        value["usage_complete"] = json!(self.complete);
    }
}

/// Old failed-style records undercounted the draft in their headline totals.
/// Read the larger known aggregate without changing saved artifacts. Preserve
/// older totals that include stages not represented in the stage table.
pub(super) fn record_totals(record: &Value) -> Usage {
    let headline = Usage::read(record);
    let Some(stages) = record["stages"].as_object() else {
        return headline;
    };
    let mut sum = None;
    for stage in stages
        .values()
        .filter(|s| s["attempts"].as_u64().unwrap_or(0) > 0)
    {
        let mut usage = Usage::read(stage);
        if usage.total == 0 && stage.get("usage_complete").is_none() {
            usage.complete = false;
        }
        sum = Some(sum.map_or(usage, |sum: Usage| sum.add(usage)));
    }
    match sum {
        Some(sum) if sum.total >= headline.total => sum,
        _ => headline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_failure_totals_include_draft_and_style_without_double_counting()
    {
        let old = json!({"prompt_tokens":20,"completion_tokens":5,"total_tokens":25,
            "stages":{"draft":{"attempts":2,"prompt_tokens":40,"completion_tokens":10},
            "style":{"attempts":3,"prompt_tokens":20,"completion_tokens":5}}});
        let total = record_totals(&old);
        assert_eq!(total.total, 75);
        assert!(total.complete);
        assert_eq!(old["total_tokens"], 25);
    }

    #[test]
    fn transport_failure_keeps_known_usage_but_marks_the_total_incomplete() {
        let usage = Usage::read(
            &json!({"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}),
        )
        .add(Usage::default());
        assert_eq!(usage.total, 15);
        assert!(!usage.complete);
    }
}
