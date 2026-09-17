//! Small-batch experiment facility: a fixed, diverse 20-system test
//! set, prepared and generated into label-suffixed variant files so
//! prompt and gate changes can be evaluated without touching the
//! served `description.md`.
use std::path::PathBuf;

use anyhow::Result;
use serde_json::{Value, json};

use super::{batch, prepare};

/// The fixed experiment set: the 15-system baseline plus five diversity
/// picks (ultra-short periods, compact multis, a resonant chain, a
/// young multi, and a bright two-planet system).
pub const EXPERIMENT_SYSTEMS: &[&str] = &[
    "51 Peg",
    "HD 41004 A",
    "Kepler-11",
    "LHS 1140",
    "TRAPPIST-1",
    "HR 8799",
    "PSR B1257+12",
    "Kepler-16",
    "55 Cnc",
    "Proxima Cen",
    "HD 189733",
    "GJ 1214",
    "K2-18",
    "HD 10180",
    "OGLE-2016-BLG-1195L",
    "Kepler-42",
    "L 98-59",
    "TOI-178",
    "V1298 Tau",
    "HD 260655",
];

pub struct Options {
    pub data_dir: PathBuf,
    pub input_dir: PathBuf,
    pub label: String,
    pub force: bool,
    pub concurrency: usize,
    pub max_tokens: u32,
}

pub fn columns() -> Vec<String> {
    batch::columns()
}

/// Compact single-line outcome for the `lines` output format.
pub fn line(row: &Value) -> String {
    batch::line(row)
}

/// Refresh the fixed set's requests (experiments always run on current
/// preparation semantics), then generate every system into
/// `description_<label>.md` variant files.
pub fn run(options: &Options) -> Result<Vec<Value>> {
    batch::validate_label(&options.label)?;
    let content_dir = options.input_dir.clone();
    eprintln!(
        "Preparing the {} experiment systems",
        EXPERIMENT_SYSTEMS.len()
    );
    let catalog = prepare::Catalog::load(&options.data_dir)?;
    for hostname in EXPERIMENT_SYSTEMS {
        eprintln!("Preparing {hostname}");
        catalog.prepare(&content_dir, hostname, true, false)?;
    }
    let mut rows = batch::run(&batch::Options {
        input_dir: content_dir,
        hostnames: EXPERIMENT_SYSTEMS.iter().map(|s| (*s).to_owned()).collect(),
        system_prompt: PathBuf::from("content/stellarhost_prompt.txt"),
        repair_prompt: PathBuf::from("content/stellarhost_repair_prompt.txt"),
        concurrency: options.concurrency,
        max_tokens: options.max_tokens,
        force: options.force,
        failed: false,
        label: Some(options.label.clone()),
    })?;
    for row in &mut rows {
        if row["status"] == "generated" {
            row["variant"] = json!(format!("description_{}.md", options.label));
        }
    }
    let written: Vec<&str> = rows
        .iter()
        .filter(|row| row["status"] == "generated")
        .filter_map(|row| row["hostname"].as_str())
        .collect();
    eprintln!(
        "experiment {}: {} generated into description_{}.md files",
        options.label,
        written.len(),
        options.label
    );
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experiment_set_is_distinct_and_sized() {
        assert_eq!(EXPERIMENT_SYSTEMS.len(), 20);
        let mut sorted = EXPERIMENT_SYSTEMS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), EXPERIMENT_SYSTEMS.len());
    }
}
