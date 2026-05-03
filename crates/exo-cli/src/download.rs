use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};

use crate::api::ApiClient;

#[derive(Clone, Copy, Debug)]
pub enum DownloadTarget {
    StellarHosts,
    Exoplanets,
    All,
}

pub fn download(
    client: &ApiClient,
    target: DownloadTarget,
    directory: &str,
    force: bool,
) -> Result<()> {
    let files = match target {
        DownloadTarget::StellarHosts => stellarhosts_files(),
        DownloadTarget::Exoplanets => exoplanets_files(),
        DownloadTarget::All => {
            let mut files = stellarhosts_files();
            files.extend(exoplanets_files());
            files
        }
    };

    let directory = Path::new(directory);
    fs::create_dir_all(directory)
        .with_context(|| format!("failed to create {}", directory.display()))?;

    for file in files {
        let destination = directory.join(file.name);
        if destination.exists() && !force {
            println!("exists: {}", destination.display());
            continue;
        }

        let progress = ProgressBar::new_spinner();
        progress.set_style(
            ProgressStyle::with_template("{spinner} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        progress.set_message(format!("downloading {}", file.source_path));
        progress.enable_steady_tick(std::time::Duration::from_millis(80));

        let bytes = client.download_file(file.source_path)?;
        if bytes.is_empty() {
            return Err(anyhow!("downloaded empty file {}", file.source_path));
        }
        fs::write(&destination, bytes).with_context(|| {
            format!("failed to write {}", destination.display())
        })?;
        progress.finish_and_clear();
        println!("wrote {}", destination.display());
    }

    Ok(())
}

struct DownloadFile {
    source_path: &'static str,
    name: &'static str,
}

fn stellarhosts_files() -> Vec<DownloadFile> {
    vec![
        DownloadFile {
            source_path: "/data/stellarhosts.parquet",
            name: "stellarhosts.parquet",
        },
        DownloadFile {
            source_path: "/data/stellarhosts-metadata.toml",
            name: "stellarhosts-metadata.toml",
        },
    ]
}

fn exoplanets_files() -> Vec<DownloadFile> {
    vec![
        DownloadFile {
            source_path: "/data/exoplanets.parquet",
            name: "exoplanets.parquet",
        },
        DownloadFile {
            source_path: "/data/exoplanets-metadata.toml",
            name: "exoplanets-metadata.toml",
        },
    ]
}
