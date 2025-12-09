# CLI Specification: exo-cli

This document describes the `exo-cli` command-line tool for managing and analyzing exoplanet catalog data.

## Overview

`exo-cli` is a standalone command-line tool built with [clap](https://github.com/clap-rs/clap) that provides:
- Data inspection and exploration
- Statistical analysis and visualization
- Data format conversion (VOTable → Parquet)
- Pretty-printed tables and charts in the terminal

The CLI uses `exo-core` for all data processing operations.

## Binary Name

The compiled binary is named `exo`:
```bash
cargo build --package exo-cli
# Output: target/release/exo
```

## Installation

**Development:**
```bash
cargo run --package exo-cli -- <command>
```

**Production:**
```bash
cargo build --package exo-cli --release
./target/release/exo <command>
```

**Install to system:**
```bash
cargo install --path crates/exo-cli
exo <command>
```

## Dependencies

```toml
exo-core = { path = "../exo-core" }
clap = { version = "4.5.52", features = ["derive"] }
comfy-table = "7.2.1"
ratatui = { version = "0.29.0", features = ["all-widgets"] }
crossterm = "0.28"
indicatif = "0.18.3"
anyhow = "1.0.100"
polars = { version = "0.52.0", features = ["lazy", "temporal", "timezones", "parquet"] }
```

## Command Structure

```
exo
├── view-fields              View VOTable structure
├── view-samples             View stellarhosts data samples
├── view-stats               View stellarhosts statistics
├── view-exoplanets-samples  View exoplanets data samples
├── view-exoplanets-stats    View exoplanets statistics
└── convert-raw-files        Convert VOTable files to Parquet
```

## Commands

### 1. view-fields

Display field definitions from a VOTable file.

**Usage:**
```bash
exo view-fields <path>
```

**Arguments:**
- `path` - Path to VOTable (.vot) file

**Example:**
```bash
exo view-fields data/stellarhosts.vot
```

**Output:**
```
FIELD. name: hostname; datatype: char
FIELD. name: hd_name; datatype: char
FIELD. name: hip_name; datatype: char
FIELD. name: st_teff; datatype: double
...
```

**Implementation:**
- Calls `exo_core::common::print_votable_headers()`
- Reads VOTable structure without loading all data
- Fast operation, suitable for large files

---

### 2. view-samples

View sample rows from stellarhosts parquet file.

**Usage:**
```bash
exo view-samples [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Path to parquet file (default: `data/stellarhosts.parquet`)
- `-l, --limit <LIMIT>` - Number of rows to show (default: 10)
- `-c, --category <CATEGORY>` - Column category to display

**Categories:**
- `basic` - hostname, hd_name, hip_name, tic_id
- `position` - ra, dec, rastr, decstr, glon, glat
- `stellar` - st_teff, st_mass, st_rad, st_logg, st_lum, st_age, st_met
- `photometry` - sy_vmag, sy_bmag, sy_jmag, sy_hmag, sy_kmag, sy_gmag
- (default) - Mixed set of important columns

**Examples:**
```bash
# View 10 rows with default columns
exo view-samples

# View 20 rows
exo view-samples -l 20

# View basic information
exo view-samples -c basic

# Custom file and category
exo view-samples -p data/custom.parquet -c stellar -l 5
```

**Output:**
Pretty-printed table using `comfy-table` with:
- Column headers
- Formatted values
- Dataset summary (total rows/columns)

**Implementation:**
- Calls `commands::view_stellarhosts_samples()`
- Loads only requested rows (efficient)
- Handles missing columns gracefully

---

### 3. view-stats

Display comprehensive statistics for stellarhosts dataset.

**Usage:**
```bash
exo view-stats [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Path to parquet file (default: `data/stellarhosts.parquet`)

**Example:**
```bash
exo view-stats
exo view-stats -p data/stellarhosts.parquet
```

**Output:**
```
Stellarhosts Data Statistics
===========================
Total rows: 5000
Total columns: 136

+-------------+-------+----------+----------+---------+----------+-----------+
| Column      | Count | Mean     | Median   | Std Dev | Min      | Max       |
+-------------+-------+----------+----------+---------+----------+-----------+
| Teff (K)    | 5000  | 5456.000 | 5593.000 | 831.521 | 2703.000 | 29300.000 |
| Mass (M☉)   | 5000  | 0.954    | 0.955    | 0.302   | 0.029    | 8.760     |
| Radius (R☉) | 5000  | 1.248    | 0.946    | 2.719   | 0.118    | 78.125    |
...

Temperature Distribution (K):
[ASCII histogram visualization]
```

**Features:**
- Summary statistics for key numeric columns
- ASCII histogram visualizations
- Formatted with units (K, M☉, R☉, etc.)

**Implementation:**
- Calls `commands::view_stellarhosts_stats()`
- Uses `exo_core::tables::common::get_numeric_stats()`
- Uses `exo_core::tables::common::create_histogram()`

---

### 4. view-exoplanets-samples

View sample rows from exoplanets parquet file.

**Usage:**
```bash
exo view-exoplanets-samples [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Path to parquet file (default: `data/exoplanets.parquet`)
- `-l, --limit <LIMIT>` - Number of rows to show (default: 10)
- `-c, --category <CATEGORY>` - Column category to display

**Categories:**
- `basic` - pl_name, hostname, discoverymethod, disc_year
- `discovery` - discoverymethod, disc_year, disc_facility
- `orbital` - pl_orbper, pl_orbsmax, pl_orbeccen, pl_orbincl
- `physical` - pl_rade, pl_bmasse, pl_dens, pl_eqt
- (default) - Mixed set of important columns

**Examples:**
```bash
# View 10 rows with default columns
exo view-exoplanets-samples

# View orbital parameters
exo view-exoplanets-samples -c orbital -l 20

# View physical properties
exo view-exoplanets-samples -c physical
```

**Output:**
Similar to `view-samples` but for exoplanets data.

---

### 5. view-exoplanets-stats

Display comprehensive statistics for exoplanets dataset.

**Usage:**
```bash
exo view-exoplanets-stats [OPTIONS]
```

**Options:**
- `-p, --path <PATH>` - Path to parquet file (default: `data/exoplanets.parquet`)

**Example:**
```bash
exo view-exoplanets-stats
```

**Output:**
Statistics for exoplanet properties:
- Planet radius distribution
- Mass distribution
- Orbital period statistics
- Discovery method breakdown
- Discovery year histogram

---

### 6. convert-raw-files

Convert all VOTable (.vot) files in a directory to Parquet format.

**Usage:**
```bash
exo convert-raw-files [OPTIONS]
```

**Options:**
- `-d, --data-dir <DATA_DIR>` - Directory containing .vot files (default: `data`)

**Example:**
```bash
# Convert all files in data/ directory
exo convert-raw-files

# Custom directory
exo convert-raw-files -d /path/to/votables
```

**Behavior:**
- Scans directory for `*.vot` files
- Converts each to `.parquet` with same base name
- Shows progress bar for each file
- Skips files on error, continues with others
- Reports summary at end

**Output:**
```
Converting VOTable files in: data
Found 2 VOTable files

Converting stellarhosts.vot...
[████████████████████] 100% (5.2s)
✓ Wrote data/stellarhosts.parquet (5000 rows, 136 columns)

Converting exoplanets.vot...
[████████████████████] 100% (3.8s)
✓ Wrote data/exoplanets.parquet (4523 rows, 98 columns)

Conversion complete: 2 files processed
```

**Implementation:**
- Calls `exo_core::tables::conversion::convert_raw_files()`
- Uses `indicatif` for progress bars
- Handles large files efficiently with streaming

---

## Implementation Details

### Module Structure

```
crates/exo-cli/src/
├── main.rs      # CLI definition and command dispatch
└── commands.rs  # Command implementations
```

### main.rs

Defines CLI structure using clap:
```rust
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    ViewFields { path: String },
    ViewSamples { ... },
    // ...
}
```

Dispatches to command implementations:
```rust
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::ViewFields { path } => {
            exo_core::common::print_votable_headers(&path);
        }
        Commands::ViewSamples { path, limit, category } => {
            commands::view_stellarhosts_samples(&path, limit, cat)?;
        }
        // ...
    }
}
```

### commands.rs

Implements display logic for each command:
- Loads data using `exo_core`
- Formats output with `comfy-table`
- Handles errors with user-friendly messages
- Uses `polars` for DataFrame operations

## Error Handling

All commands handle errors gracefully:
```rust
if let Err(e) = commands::view_stats(&path) {
    eprintln!("Error viewing stats: {}", e);
}
```

Common errors:
- File not found
- Invalid parquet format
- Column not found
- Parse errors

## Future Commands

Planned additions:
- `exo diff` - Compare two datasets
- `exo validate` - Validate data integrity
- `exo export` - Export to CSV/JSON
- `exo query` - SQL-like queries on data
- `exo filter` - Filter data by criteria
- `exo merge` - Merge multiple datasets
- `exo tui` - Interactive TUI mode

## Performance

- **Fast startup**: Binary size ~15MB (release)
- **Low memory**: Only loads requested data
- **Streaming**: Large files processed efficiently
- **Progress bars**: User feedback for long operations

## Best Practices

1. **Use limits for exploration**: `exo view-samples -l 100`
2. **Convert to parquet first**: Faster than reading VOTable repeatedly
3. **Use categories**: View only relevant columns
4. **Pipe output**: Combine with other CLI tools (`grep`, `awk`, etc.)

## Examples Workflow

```bash
# 1. Download raw VOTable data
just download-stellarhosts

# 2. Convert to Parquet
exo convert-raw-files

# 3. Explore structure
exo view-fields data/stellarhosts.vot

# 4. View samples
exo view-samples -c stellar -l 20

# 5. Get statistics
exo view-stats

# 6. Analyze exoplanets
exo view-exoplanets-samples -c orbital
exo view-exoplanets-stats
```
