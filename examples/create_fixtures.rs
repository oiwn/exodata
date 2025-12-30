use polars::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs;
use std::path::Path;

// Configuration constants
const DATA_DIR: &str = "data";
const OUTPUT_DIR: &str = "crates/exo-core/tests/fixtures";
const SAMPLE_SIZE: usize = 100;
const RANDOM_SEED: u64 = 42;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("VOTable Fixture Generator");
    println!("=========================");
    println!("Data directory: {}", DATA_DIR);
    println!("Output directory: {}", OUTPUT_DIR);
    println!("Sample size: {}", SAMPLE_SIZE);
    println!("Random seed: {}", RANDOM_SEED);
    println!();

    // Create output directory if it doesn't exist
    fs::create_dir_all(OUTPUT_DIR)
        .expect(format!("Unable to create directory: {}", OUTPUT_DIR).as_str());

    // Find all .vot files in data directory
    let votable_files = discover_votable_files(DATA_DIR)?;

    if votable_files.is_empty() {
        println!("No VOTable files found in {}", DATA_DIR);
        return Ok(());
    }

    println!("Found {} VOTable file(s):", votable_files.len());
    for file in &votable_files {
        println!("  - {}", file);
    }
    println!();

    // Process each VOTable file
    for votable_path in votable_files {
        process_votable(&votable_path);
    }

    println!("\n✓ All fixtures generated successfully!");
    Ok(())
}

/// Discover all .vot files in the given directory
fn discover_votable_files(
    data_dir: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut votable_files = Vec::new();

    let entries = fs::read_dir(data_dir)
        .expect(format!("Failed to read directory: {}", data_dir).as_str());

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "vot" {
                    if let Some(path_str) = path.to_str() {
                        votable_files.push(path_str.to_string());
                    }
                }
            }
        }
    }

    votable_files.sort();
    Ok(votable_files)
}

/// Process a single VOTable file and generate fixture
fn process_votable(votable_path: &str) {
    println!("Processing: {}", votable_path);

    // Load the VOTable
    let df = exo_core::tables::votable_loader::load_votable(votable_path, None)
        .expect(format!("Failed to load VOTable: {}", votable_path).as_str());

    let total_rows = df.height();
    println!("  Total rows: {}", total_rows);
    println!("  Total columns: {}", df.width());

    // Sample the data with fixed seed
    let actual_sample_size = std::cmp::min(SAMPLE_SIZE, total_rows);
    let sampled = if total_rows > SAMPLE_SIZE {
        // Generate random indices for sampling
        let mut rng = StdRng::seed_from_u64(RANDOM_SEED);
        let mut indices = Vec::new();
        let mut used = std::collections::HashSet::new();

        while indices.len() < actual_sample_size {
            let idx = rng.random_range(0..total_rows);
            if used.insert(idx) {
                indices.push(idx as u32);
            }
        }

        indices.sort();
        let indices_ca =
            UInt32Chunked::from_vec(PlSmallStr::from("idx"), indices);
        df.take(&indices_ca).expect("Failed to sample DataFrame")
    } else {
        df.clone()
    };

    println!("  Sampled rows: {}", sampled.height());

    // Extract basename for output files
    let path = Path::new(votable_path);
    let basename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Invalid file path");

    // Save as .fixture file (Parquet format)
    let fixture_path = format!("{}/{}.fixture", OUTPUT_DIR, basename);
    let mut file = fs::File::create(&fixture_path).expect(
        format!("Failed to create fixture file: {}", fixture_path).as_str(),
    );

    ParquetWriter::new(&mut file)
        .finish(&mut sampled.clone())
        .expect("Failed to write Parquet fixture");

    println!("  ✓ Created: {}", fixture_path);

    // Save metadata as JSON
    let metadata = serde_json::json!({
        "source": votable_path,
        "sample_size": actual_sample_size,
        "total_rows": total_rows,
        "seed": RANDOM_SEED,
        "rows": sampled.height(),
        "columns": sampled.width(),
        "column_names": sampled.get_column_names(),
        "dtypes": sampled.dtypes().iter().map(|dt| dt.to_string()).collect::<Vec<_>>(),
    });

    let json_path = format!("{}/{}.fixture.json", OUTPUT_DIR, basename);
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&metadata).expect("Failed to prettify json"),
    )
    .expect(format!("Failed to write metadata: {}", json_path).as_str());

    println!("  ✓ Created: {}", json_path);
    println!();
}
