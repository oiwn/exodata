use std::path::Path;

use anyhow::{Error, anyhow};
use comfy_table::{
    ContentLineStyle, LineStyle, Table, TableStyle, presets::ASCII_MARKDOWN,
};
use exo_core::insights::{self, InsightInput};
use exo_core::tables::common::{
    create_histogram, get_numeric_stats, load_data_with_limit, load_parquet,
    print_histogram,
};
use polars::prelude::*;
use polars::sql::SQLContext;

const TERMINAL_TABLE: TableStyle = TableStyle::new()
    .top_border(LineStyle::new('+', '-', '+', '+'))
    .header_lines(ContentLineStyle::new('|', '|', '|'))
    .header_separator(LineStyle::new('+', '-', '+', '+'))
    .content_lines(ContentLineStyle::new('|', '|', '|'))
    .bottom_border(LineStyle::new('+', '-', '+', '+'));

/// Display sample data from stellarhosts table
pub fn view_stellarhosts_samples(
    path: &Path,
    limit: Option<usize>,
    category: Option<&str>,
) -> Result<(), Error> {
    // If no limit specified, default to 10
    let limit = limit.unwrap_or(10);

    // Load only rows we need (plus a small buffer for any null rows)
    let load_limit = Some(limit + 10);
    let df = load_data_with_limit(path.to_str().unwrap(), load_limit)?;
    let limit = std::cmp::min(limit, df.height());

    // Get first few rows as a preview
    let preview_df = df.slice(0, limit);

    // Define column categories based on field names we observed
    let basic_cols = ["hostname", "hd_name", "hip_name", "tic_id"];
    let position_cols = ["ra", "dec", "rastr", "decstr", "glon", "glat"];
    let stellar_props_cols = vec![
        "st_teff", "st_mass", "st_rad", "st_logg", "st_lum", "st_age", "st_met",
        "st_radv", "st_vsin",
    ];
    let photometry_cols = [
        "sy_vmag",
        "sy_bmag",
        "sy_jmag",
        "sy_hmag",
        "sy_kmag",
        "sy_gmag",
        "sy_gaiamag",
        "sy_kepmag",
    ];

    // Select which columns to display based on category
    let cols_to_show = match category {
        Some("basic") => &basic_cols[..],
        Some("position") => &position_cols[..],
        Some("stellar") => &stellar_props_cols[..],
        Some("photometry") => &photometry_cols[..],
        _ => {
            // Default: show a mix of important columns
            &[
                "hostname",
                "ra",
                "dec",
                "st_teff",
                "st_mass",
                "st_rad",
                "st_logg",
                "sy_vmag",
                "sy_gaiamag",
            ][..]
        }
    };

    // Filter to only include columns that exist in dataframe
    let available_cols: Vec<String> = cols_to_show
        .iter()
        .filter(|&&col| {
            preview_df
                .get_column_names()
                .into_iter()
                .any(|name| name.as_str() == col)
        })
        .map(|&col| col.to_string())
        .collect();

    if available_cols.is_empty() {
        println!("No columns available to display");
        return Ok(());
    }

    // Create a selection with available columns
    let selected_df = preview_df.select(&available_cols)?;

    // Display information about dataset
    println!("Stellarhosts Data Sample");
    println!("========================");
    println!("Total rows in dataset: {}", df.height());
    println!("Total columns in dataset: {}", df.width());
    println!(
        "Showing {} rows of category: {:?}",
        limit,
        category.unwrap_or("mixed")
    );
    println!();

    // Create and configure table
    let mut table = Table::new();
    table.load_style(TERMINAL_TABLE);

    // Add header row
    let headers: Vec<String> = available_cols
        .iter()
        .map(|col| {
            // Format column names to be more readable
            match col.as_str() {
                "hostname" => "Host Name".to_string(),
                "hd_name" => "HD Name".to_string(),
                "hip_name" => "HIP Name".to_string(),
                "tic_id" => "TIC ID".to_string(),
                "ra" => "RA (deg)".to_string(),
                "dec" => "Dec (deg)".to_string(),
                "rastr" => "RA (str)".to_string(),
                "decstr" => "Dec (str)".to_string(),
                "glon" => "Gal Lon".to_string(),
                "glat" => "Gal Lat".to_string(),
                "st_teff" => "Teff (K)".to_string(),
                "st_mass" => "Mass (M☉)".to_string(),
                "st_rad" => "Radius (R☉)".to_string(),
                "st_logg" => "log(g)".to_string(),
                "st_lum" => "Luminosity".to_string(),
                "st_age" => "Age (Gyr)".to_string(),
                "st_met" => "Metallicity".to_string(),
                "st_radv" => "Radial Vel".to_string(),
                "st_vsin" => "v sin(i)".to_string(),
                "sy_vmag" => "V Mag".to_string(),
                "sy_bmag" => "B Mag".to_string(),
                "sy_jmag" => "J Mag".to_string(),
                "sy_hmag" => "H Mag".to_string(),
                "sy_kmag" => "K Mag".to_string(),
                "sy_gmag" => "G Mag".to_string(),
                "sy_gaiamag" => "Gaia Mag".to_string(),
                "sy_kepmag" => "Kepler Mag".to_string(),
                _ => col.clone(),
            }
        })
        .collect();

    table.set_header(headers);

    // Add data rows
    for row_idx in 0..selected_df.height() {
        let mut row_data = Vec::new();

        for col_name in &available_cols {
            let col = selected_df.column(col_name)?;
            let value = match col.get(row_idx) {
                Ok(AnyValue::Null) => "N/A".to_string(),
                Ok(AnyValue::Float64(v)) => format!("{:.3}", v),
                Ok(AnyValue::Float32(v)) => format!("{:.3}", v),
                Ok(AnyValue::Int64(v)) => format!("{}", v),
                Ok(AnyValue::Int32(v)) => format!("{}", v),
                Ok(AnyValue::Int16(v)) => format!("{}", v),
                Ok(AnyValue::String(v)) => v.to_string(),
                Ok(_) => "Unknown".to_string(),
                Err(_) => "Error".to_string(),
            };
            row_data.push(value);
        }

        table.add_row(row_data);
    }

    // Print the table
    println!("{}", table);

    // Display available categories if a specific one wasn't requested
    if category.is_none() {
        println!();
        println!("Available data categories:");
        println!("  basic       - Host names and identifiers");
        println!("  position    - Sky coordinates (RA, Dec, Galactic)");
        println!(
            "  stellar     - Stellar properties (mass, radius, temperature, etc.)"
        );
        println!("  photometry  - Magnitudes in various photometric bands");
        println!();
        println!("Usage examples:");
        println!("  view-stellarhosts-samples --limit 5 --category basic");
        println!("  view-stellarhosts-samples --category position");
    }

    Ok(())
}

/// Display summary statistics for stellarhosts data
pub fn view_stellarhosts_stats(path: &str) -> Result<(), Error> {
    let df = load_data_with_limit(path, Some(5000))?; // Load a sample for faster stats

    println!("Stellarhosts Data Statistics");
    println!("===========================");
    println!("Total rows: {}", df.height());
    println!("Total columns: {}", df.width());
    println!();

    // Display basic statistics for important columns
    display_basic_stats(
        &df,
        &[
            ("st_teff", "Teff (K)"),
            ("st_mass", "Mass (M☉)"),
            ("st_rad", "Radius (R☉)"),
            ("st_logg", "log(g)"),
            ("st_lum", "Luminosity"),
            ("st_age", "Age (Gyr)"),
            ("st_met", "Metallicity"),
            ("st_radv", "Radial Vel"),
            ("st_vsin", "v sin(i)"),
            ("sy_dist", "Distance"),
            ("ra", "RA (deg)"),
            ("dec", "Dec (deg)"),
            ("glon", "Gal Lon"),
            ("glat", "Gal Lat"),
        ],
    )?;

    // Show distributions for key stellar properties
    println!();
    println!("Temperature Distribution (K):");
    let temp_hist = create_histogram(&df, "st_teff", 3000.0, 10000.0, 7)?;
    print_histogram(&temp_hist, 50);

    println!("\nMass Distribution (Solar masses):");
    let mass_hist = create_histogram(&df, "st_mass", 0.1, 3.0, 6)?;
    print_histogram(&mass_hist, 50);

    println!("\nRadius Distribution (Solar radii):");
    let rad_hist = create_histogram(&df, "st_rad", 0.1, 5.0, 5)?;
    print_histogram(&rad_hist, 50);

    Ok(())
}

/// Display sample data from exoplanets table
pub fn view_exoplanets_samples(
    path: &str,
    limit: Option<usize>,
    category: Option<&str>,
) -> Result<(), Error> {
    // If no limit specified, default to 10
    let limit = limit.unwrap_or(10);

    // Load only rows we need (plus a small buffer for any null rows)
    let load_limit = Some(limit + 10);
    let df = load_data_with_limit(path, load_limit)?;
    let limit = std::cmp::min(limit, df.height());

    // Get first few rows as a preview
    let preview_df = df.slice(0, limit);

    // Define column categories based on exoplanet fields we observed
    let basic_cols = ["pl_name", "pl_letter", "hostname", "disc_year"];
    let discovery_cols = [
        "discoverymethod",
        "disc_facility",
        "disc_instrument",
        "disc_telescope",
    ];
    let orbital_cols = ["pl_orbper", "pl_orbsmax", "pl_orbeccen", "pl_eqt"];
    let physical_cols = ["pl_massj", "pl_masse", "pl_rade", "pl_radj"];

    // Select which columns to display based on category
    let cols_to_show = match category {
        Some("basic") => &basic_cols[..],
        Some("discovery") => &discovery_cols[..],
        Some("orbital") => &orbital_cols[..],
        Some("physical") => &physical_cols[..],
        _ => {
            // Default: show a mix of important columns
            &[
                "pl_name",
                "hostname",
                "pl_orbper",
                "pl_masse",
                "pl_rade",
                "disc_year",
            ][..]
        }
    };

    // Filter to only include columns that exist in dataframe
    let available_cols: Vec<String> = cols_to_show
        .iter()
        .filter(|&&col| {
            preview_df
                .get_column_names()
                .into_iter()
                .any(|name| name.as_str() == col)
        })
        .map(|&col| col.to_string())
        .collect();

    if available_cols.is_empty() {
        println!("No columns available to display");
        return Ok(());
    }

    // Create a selection with available columns
    let selected_df = preview_df.select(&available_cols)?;

    // Display information about dataset
    println!("Exoplanets Data Sample");
    println!("======================");
    println!("Total rows in dataset: {}", df.height());
    println!("Total columns in dataset: {}", df.width());
    println!(
        "Showing {} rows of category: {:?}",
        limit,
        category.unwrap_or("mixed")
    );
    println!();

    // Create and configure table
    let mut table = Table::new();
    table.load_style(TERMINAL_TABLE);

    // Add header row with formatted names
    let headers: Vec<String> = available_cols
        .iter()
        .map(|col| {
            // Format column names to be more readable
            match col.as_str() {
                "pl_name" => "Planet Name".to_string(),
                "pl_letter" => "Letter".to_string(),
                "hostname" => "Host Name".to_string(),
                "disc_year" => "Disc Year".to_string(),
                "discoverymethod" => "Method".to_string(),
                "disc_facility" => "Facility".to_string(),
                "disc_instrument" => "Instrument".to_string(),
                "disc_telescope" => "Telescope".to_string(),
                "pl_orbper" => "Orbital Period".to_string(),
                "pl_orbsmax" => "Semi-Major Axis".to_string(),
                "pl_orbeccen" => "Eccentricity".to_string(),
                "pl_eqt" => "Equilibrium Temp".to_string(),
                "pl_massj" => "Mass (M_J)".to_string(),
                "pl_masse" => "Mass (M_E)".to_string(),
                "pl_rade" => "Radius (R_J)".to_string(),
                "pl_radj" => "Radius (R_E)".to_string(),
                _ => col.clone(),
            }
        })
        .collect();

    table.set_header(headers);

    // Add data rows
    for row_idx in 0..selected_df.height() {
        let mut row_data = Vec::new();

        for col_name in &available_cols {
            let col = selected_df.column(col_name)?;
            let value = match col.get(row_idx) {
                Ok(AnyValue::Null) => "N/A".to_string(),
                Ok(AnyValue::Float64(v)) => format!("{:.3}", v),
                Ok(AnyValue::Float32(v)) => format!("{:.3}", v),
                Ok(AnyValue::Int64(v)) => format!("{}", v),
                Ok(AnyValue::Int32(v)) => format!("{}", v),
                Ok(AnyValue::Int16(v)) => format!("{}", v),
                Ok(AnyValue::String(v)) => v.to_string(),
                Ok(_) => "Unknown".to_string(),
                Err(_) => "Error".to_string(),
            };
            row_data.push(value);
        }

        table.add_row(row_data);
    }

    // Print the table
    println!("{}", table);

    // Display available categories if a specific one wasn't requested
    if category.is_none() {
        println!();
        println!("Available data categories:");
        println!("  basic       - Planet names and basic info");
        println!("  discovery   - Discovery information");
        println!("  orbital     - Orbital parameters");
        println!("  physical    - Physical properties (mass, radius)");
        println!();
        println!("Usage examples:");
        println!("  view-exoplanets-samples --limit 5 --category basic");
        println!("  view-exoplanets-samples --category orbital");
    }

    Ok(())
}

/// Display summary statistics for exoplanets data
pub fn view_exoplanets_stats(path: &str) -> Result<(), Error> {
    let df = load_data_with_limit(path, Some(5000))?; // Load a sample for faster stats

    println!("Exoplanets Data Statistics");
    println!("==========================");
    println!("Total rows: {}", df.height());
    println!("Total columns: {}", df.width());
    println!();

    // Display basic statistics for important columns
    display_basic_stats(
        &df,
        &[
            ("pl_masse", "Mass (M_E)"),
            ("pl_massj", "Mass (M_J)"),
            ("pl_rade", "Radius (R_J)"),
            ("pl_radj", "Radius (R_E)"),
            ("pl_orbper", "Orbital Period"),
            ("pl_orbsmax", "Semi-Major Axis"),
            ("pl_orbeccen", "Eccentricity"),
            ("pl_eqt", "Equilibrium Temp"),
            ("ra", "RA (deg)"),
            ("dec", "Dec (deg)"),
            ("disc_year", "Discovery Year"),
        ],
    )?;

    // Show distributions for key planetary properties
    println!();
    println!("Mass Distribution (Earth masses):");
    let mass_hist = create_histogram(&df, "pl_masse", 0.0, 1000.0, 8)?;
    print_histogram(&mass_hist, 50);

    println!("\nOrbital Period Distribution (days):");
    let period_hist = create_histogram(&df, "pl_orbper", 0.1, 1000.0, 8)?;
    print_histogram(&period_hist, 50);

    println!("\nRadius Distribution (Jupiter radii):");
    let rad_hist = create_histogram(&df, "pl_rade", 0.1, 5.0, 6)?;
    print_histogram(&rad_hist, 50);

    Ok(())
}

/// Display basic statistics table
fn display_basic_stats(
    df: &DataFrame,
    key_columns: &[(&str, &str)],
) -> Result<(), Error> {
    let mut table = Table::new();
    table.load_style(TERMINAL_TABLE);
    table.set_header(vec![
        "Column", "Count", "Mean", "Median", "Std Dev", "Min", "Max",
    ]);

    for (col_name, display_name) in key_columns {
        if let Some(stats) = get_numeric_stats(df, col_name)? {
            let row: Vec<String> = vec![
                display_name.to_string(),
                stats.count.to_string(),
                format!("{:.3}", stats.mean),
                format!("{:.3}", stats.median),
                format!("{:.3}", stats.std),
                format!("{:.3}", stats.min),
                format!("{:.3}", stats.max),
            ];
            table.add_row(row);
        }
    }

    println!("{}", table);
    Ok(())
}

/// Execute SQL query against parquet files
pub fn execute_sql(query: &str, data_dir: &str) -> Result<(), Error> {
    let result = execute_sql_frame(query, data_dir)?;
    println!("{}", result);
    Ok(())
}

pub fn execute_sql_frame(
    query: &str,
    data_dir: &str,
) -> Result<DataFrame, Error> {
    let stellarhosts_path = format!("{}/stellarhosts.parquet", data_dir);
    let exoplanets_path = format!("{}/exoplanets.parquet", data_dir);

    let mut ctx = SQLContext::new();

    if Path::new(&stellarhosts_path).exists() {
        let df = load_parquet(&stellarhosts_path, None)?;
        ctx.register("stellarhosts", df.lazy());
    }

    if Path::new(&exoplanets_path).exists() {
        let df = load_parquet(&exoplanets_path, None)?;
        ctx.register("exoplanets", df.lazy());
    }

    ctx.execute(query)
        .map_err(|e| anyhow::anyhow!("SQL error: {}", e))?
        .collect()
        .map_err(|e| anyhow::anyhow!("Failed to collect result: {}", e))
}

pub fn rows_frame(
    table: &str,
    data_dir: &str,
    columns: Option<&str>,
    sort_by: Option<&str>,
    order: Option<&str>,
    limit: usize,
) -> Result<DataFrame, Error> {
    let selected_columns = columns.unwrap_or("*");
    let mut query = format!("SELECT {selected_columns} FROM {table}");

    if let Some(sort_by) = sort_by {
        let order = match order {
            Some(value) if value.eq_ignore_ascii_case("desc") => "DESC",
            _ => "ASC",
        };
        query.push_str(&format!(" ORDER BY {sort_by} {order}"));
    }

    query.push_str(&format!(" LIMIT {limit}"));
    execute_sql_frame(&query, data_dir)
}

/// List curated insight queries available through exo-core.
pub fn list_insights() {
    let mut table = minimal_table();
    table.set_header(vec!["Slug", "Title", "Category", "Limit"]);

    for &def in insights::INSIGHTS {
        table.add_row(vec![
            def.meta.slug.to_string(),
            def.meta.title.to_string(),
            def.meta.category.to_string(),
            def.meta.limit.to_string(),
        ]);
    }

    println!("{}", table);
}

/// Run one curated insight query by slug.
pub fn run_insight(slug: &str, data_dir: &str) -> Result<(), Error> {
    let frames = load_insight_frames(data_dir)?;
    let def = insights::find_insight(slug)
        .ok_or_else(|| anyhow!("unknown insight slug '{}'", slug))?;
    let data = insights::run_insight_def(frames.input(), def)?;

    print_insight_result(def, &data.frame)?;

    Ok(())
}

/// Run all curated insight queries in registry order.
pub fn run_all_insights(data_dir: &str) -> Result<(), Error> {
    let frames = load_insight_frames(data_dir)?;
    let mut failures = Vec::new();

    for &def in insights::INSIGHTS {
        match insights::run_insight_def(frames.input(), def) {
            Ok(data) => {
                if let Err(err) = print_insight_result(def, &data.frame) {
                    failures.push(format!("{}: {}", def.meta.slug, err));
                }
            }
            Err(err) => failures.push(format!("{}: {}", def.meta.slug, err)),
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "{} insight(s) failed:\n{}",
            failures.len(),
            failures.join("\n")
        ))
    }
}

struct InsightFrames {
    stellarhosts: DataFrame,
    exoplanets: DataFrame,
}

impl InsightFrames {
    fn input(&self) -> InsightInput<'_> {
        InsightInput {
            stellarhosts: &self.stellarhosts,
            exoplanets: &self.exoplanets,
        }
    }
}

fn load_insight_frames(data_dir: &str) -> Result<InsightFrames, Error> {
    let stellarhosts_path = format!("{}/stellarhosts.parquet", data_dir);
    let exoplanets_path = format!("{}/exoplanets.parquet", data_dir);

    Ok(InsightFrames {
        stellarhosts: load_parquet(&stellarhosts_path, None).map_err(|e| {
            anyhow!("failed to load {}: {}", stellarhosts_path, e)
        })?,
        exoplanets: load_parquet(&exoplanets_path, None)
            .map_err(|e| anyhow!("failed to load {}: {}", exoplanets_path, e))?,
    })
}

fn print_insight_result(
    def: &insights::InsightDef,
    frame: &DataFrame,
) -> Result<(), Error> {
    println!();
    println!("{}", def.meta.title);
    println!("{}", def.meta.description);
    println!("{} rows", frame.height());
    println!();
    println!("{}", dataframe_table(frame)?);

    Ok(())
}

fn dataframe_table(frame: &DataFrame) -> Result<Table, Error> {
    let columns = frame
        .get_column_names()
        .iter()
        .filter(|name| !is_link_helper_column(name.as_str()))
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let mut table = minimal_table();
    table.set_header(columns.clone());

    for row_idx in 0..frame.height() {
        let mut row = Vec::with_capacity(columns.len());

        for column in &columns {
            let value = frame
                .column(column)
                .map_err(|e| anyhow!("failed to read column {}: {}", column, e))?
                .get(row_idx)
                .map_err(|e| {
                    anyhow!("failed to read {} at row {}: {}", column, row_idx, e)
                })?;
            row.push(format_any_value(value));
        }

        table.add_row(row);
    }

    Ok(table)
}

fn is_link_helper_column(column: &str) -> bool {
    column == "host_link_hostname"
}

fn minimal_table() -> Table {
    let mut table = Table::new();
    table.load_style(ASCII_MARKDOWN);
    table
}

fn format_any_value(value: AnyValue<'_>) -> String {
    match value {
        AnyValue::Null => "N/A".to_string(),
        AnyValue::Float64(value) => format_float(value),
        AnyValue::Float32(value) => format_float(value as f64),
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Int64(value) => value.to_string(),
        AnyValue::Int32(value) => value.to_string(),
        AnyValue::Int16(value) => value.to_string(),
        AnyValue::Int8(value) => value.to_string(),
        AnyValue::UInt64(value) => value.to_string(),
        AnyValue::UInt32(value) => value.to_string(),
        AnyValue::UInt16(value) => value.to_string(),
        AnyValue::UInt8(value) => value.to_string(),
        AnyValue::Boolean(value) => value.to_string(),
        other => other.to_string(),
    }
}

fn format_float(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    let mut text = format!("{:.4}", value);
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

/// View column metadata from a VOTable file
pub fn view_metadata(path: &str, columns: Option<&str>) -> Result<(), Error> {
    use crate::votable_helpers::parse_votable_metadata;
    use exo_core::metadata;

    println!("Loading metadata from: {}\n", path);

    let all_metadata =
        parse_votable_metadata(path).map_err(anyhow::Error::msg)?;

    if let Some(col_filter) = columns {
        // Filter to specific columns
        let column_names: Vec<&str> =
            col_filter.split(',').map(|s| s.trim()).collect();

        println!(
            "Showing metadata for {} specific columns:\n",
            column_names.len()
        );

        let mut found = 0;
        for col_name in &column_names {
            if let Some(meta) = all_metadata.get(*col_name) {
                println!("Column: {}", meta.name);
                if let Some(desc) = &meta.description {
                    println!("  Description: {}", desc);
                }
                if let Some(unit) = &meta.unit {
                    println!("  Unit: {}", unit);
                }
                println!("  Data Type: {}", meta.datatype);
                println!();
                found += 1;
            } else {
                println!("Warning: Column '{}' not found in metadata", col_name);
            }
        }

        if found == 0 {
            println!("No matching columns found.");
        }
    } else {
        // Show all metadata
        metadata::print_metadata(&all_metadata);
    }

    Ok(())
}
