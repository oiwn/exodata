use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Error, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use polars::prelude::{ParquetCompression, ParquetReader, ParquetWriter, *};

use crate::votable_helpers::parse_votable_metadata;
use crate::votable_loader::load_votable_with_progress_timed;

/// Convert all `.vot` files in `data_dir` to `.parquet` in the same directory.
/// Uses the same VOTable loader as the CLI sample commands and validates that
/// the generated parquet file is readable.
pub fn convert_raw_files(data_dir: &Path) -> Result<(), Error> {
    let mut vot_files: Vec<PathBuf> = fs::read_dir(data_dir)
        .with_context(|| {
            format!("Failed to read data directory {}", data_dir.display())
        })?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("vot") {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect();

    if vot_files.is_empty() {
        println!("No .vot files found in {}", data_dir.display());
        return Ok(());
    }

    vot_files.sort();

    let mut total_rows: usize = 0;
    let mut total_cols: usize = 0;
    let mut total_vot_bytes: u64 = 0;
    let mut total_parquet_bytes: u64 = 0;
    let mut total_elapsed = Duration::ZERO;

    for path in &vot_files {
        let start = Instant::now();
        let file_name = path
            .file_name()
            .and_then(|p| p.to_str())
            .unwrap_or("unknown file");

        let row_pb = ProgressBar::new(0);
        row_pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} rows ({eta_precise}) {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        row_pb.enable_steady_tick(Duration::from_millis(150));

        row_pb.set_message(format!("Loading {}", file_name));
        row_pb.set_message("Parsing VOTable");
        let (mut df, timings) = load_votable_with_progress_timed(
            path.to_string_lossy().as_ref(),
            None,
            Some(&row_pb),
        )
        .with_context(|| format!("Failed to load {}", path.display()))?;
        let row_count = df.height();
        let col_count = df.width();
        let vot_size_bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        row_pb.finish_and_clear();

        let output_path = path.with_extension("parquet");

        row_pb.set_message(format!("Writing {}", output_path.display()));
        let file = File::create(&output_path).with_context(|| {
            format!("Failed to create {}", output_path.display())
        })?;
        ParquetWriter::new(file)
            .with_compression(ParquetCompression::Zstd(None))
            .finish(&mut df)
            .with_context(|| {
                format!("Failed to write {}", output_path.display())
            })?;

        row_pb.set_message("Extracting metadata");
        let vot_metadata =
            parse_votable_metadata(path.to_string_lossy().as_ref())
                .unwrap_or_default();

        let metadata_path = path.with_file_name(format!(
            "{}-metadata.toml",
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        ));

        row_pb.set_message(format!(
            "Writing metadata to {}",
            metadata_path.display()
        ));
        exo_core::metadata::save_metadata_toml(&vot_metadata, &metadata_path)
            .map_err(|e| {
                anyhow!(
                    "Failed to save metadata to {}: {}",
                    metadata_path.display(),
                    e
                )
            })?;

        row_pb.set_message(format!("Validating {}", output_path.display()));
        validate_parquet(&output_path, row_count, col_count)?;
        let parquet_size_bytes =
            fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

        let elapsed = start.elapsed();
        let vot_mb = vot_size_bytes as f64 / (1024.0 * 1024.0);
        let parquet_mb = parquet_size_bytes as f64 / (1024.0 * 1024.0);
        let compression = if parquet_size_bytes > 0 {
            vot_size_bytes as f64 / parquet_size_bytes as f64
        } else {
            0.0
        };
        row_pb.println(format!(
            "Converted {} -> {} (rows: {}, cols: {}, size: {:.1} MB -> {:.1} MB, compression: {:.2}x, timings: meta {:.2?}, rows {:.2?}, write+validate {:.2?}, total {:.2?})",
            file_name,
            output_path
                .file_name()
                .and_then(|p| p.to_str())
                .unwrap_or_default(),
            row_count,
            col_count,
            vot_mb,
            parquet_mb,
            compression,
            timings.metadata,
            timings.rows,
            elapsed.saturating_sub(timings.metadata + timings.rows),
            elapsed
        ));
        row_pb.finish_and_clear();

        total_rows += row_count;
        total_cols = col_count.max(total_cols);
        total_vot_bytes += vot_size_bytes;
        total_parquet_bytes += parquet_size_bytes;
        total_elapsed += elapsed;
    }

    if !vot_files.is_empty() {
        let total_vot_mb = total_vot_bytes as f64 / (1024.0 * 1024.0);
        let total_parquet_mb = total_parquet_bytes as f64 / (1024.0 * 1024.0);
        let compression = if total_parquet_bytes > 0 {
            total_vot_bytes as f64 / total_parquet_bytes as f64
        } else {
            0.0
        };
        let throughput_mb_s = if total_elapsed > Duration::ZERO {
            total_vot_mb / total_elapsed.as_secs_f64()
        } else {
            0.0
        };

        println!(
            "Summary: files={}, rows≈{}, cols(max)={}, size {:.1} MB -> {:.1} MB (compression {:.2}x), elapsed {:.2?}, throughput {:.1} MB/s",
            vot_files.len(),
            total_rows,
            total_cols,
            total_vot_mb,
            total_parquet_mb,
            compression,
            total_elapsed,
            throughput_mb_s
        );
    }
    Ok(())
}

fn validate_parquet(
    path: &Path,
    expected_rows: usize,
    expected_cols: usize,
) -> Result<(), Error> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open parquet {}", path.display()))?;
    let df = ParquetReader::new(file).finish().map_err(|e| {
        anyhow!("Parquet validation failed for {}: {}", path.display(), e)
    })?;

    if df.height() != expected_rows {
        return Err(anyhow!(
            "Validation failed for {}: row count mismatch (expected {}, got {})",
            path.display(),
            expected_rows,
            df.height()
        ));
    }

    if df.width() != expected_cols {
        return Err(anyhow!(
            "Validation failed for {}: column count mismatch (expected {}, got {})",
            path.display(),
            expected_cols,
            df.width()
        ));
    }

    Ok(())
}
