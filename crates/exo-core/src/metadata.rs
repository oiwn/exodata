use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub datatype: String,
}

/// Container for TOML file format
#[derive(Debug, Serialize, Deserialize)]
struct MetadataFile {
    column: Vec<ColumnMetadata>,
}

/// Print metadata in a human-readable format
pub fn print_metadata(metadata: &HashMap<String, ColumnMetadata>) {
    println!(
        "\n{:<25} {:<15} {:<60}",
        "Column Name", "Unit", "Description"
    );
    println!("{}", "=".repeat(100));

    let mut sorted_keys: Vec<_> = metadata.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        if let Some(meta) = metadata.get(key) {
            let unit = meta.unit.as_deref().unwrap_or("-");
            let desc = meta.description.as_deref().unwrap_or("-");
            println!("{:<25} {:<15} {:<60}", meta.name, unit, desc);
        }
    }
    println!("\nTotal columns: {}\n", metadata.len());
}

/// Get metadata for specific columns only
pub fn get_columns_metadata(
    all_metadata: &HashMap<String, ColumnMetadata>,
    column_names: &[&str],
) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for &col_name in column_names {
        if let Some(meta) = all_metadata.get(col_name) {
            let mut description = meta.description.clone().unwrap_or_default();

            // Append unit if available
            if let Some(unit) = &meta.unit {
                if !description.contains('[') && !unit.is_empty() {
                    description = format!("{} [{}]", description, unit);
                }
            }

            result.insert(col_name.to_string(), description);
        }
    }

    result
}

/// Save metadata to a TOML file
pub fn save_metadata_toml(
    metadata: &HashMap<String, ColumnMetadata>,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert HashMap to Vec for TOML serialization
    let mut columns: Vec<ColumnMetadata> = metadata.values().cloned().collect();

    // Sort by column name for consistent output
    columns.sort_by(|a, b| a.name.cmp(&b.name));

    let metadata_file = MetadataFile { column: columns };

    // Serialize to TOML string
    let toml_string = toml::to_string_pretty(&metadata_file)?;

    // Write to file
    fs::write(output_path, toml_string)?;

    Ok(())
}

/// Load metadata from a TOML file
pub fn load_metadata_toml(
    path: &Path,
) -> Result<HashMap<String, ColumnMetadata>, Box<dyn std::error::Error>> {
    // Read file contents
    let contents = fs::read_to_string(path)?;

    // Deserialize from TOML
    let metadata_file: MetadataFile = toml::from_str(&contents)?;

    // Convert Vec to HashMap for O(1) lookups
    let metadata_map: HashMap<String, ColumnMetadata> = metadata_file
        .column
        .into_iter()
        .map(|m| (m.name.clone(), m))
        .collect();

    Ok(metadata_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_columns_metadata() {
        let mut all_meta = HashMap::new();
        all_meta.insert(
            "pl_name".to_string(),
            ColumnMetadata {
                name: "pl_name".to_string(),
                description: Some("Planet Name".to_string()),
                unit: None,
                datatype: "char".to_string(),
            },
        );
        all_meta.insert(
            "pl_orbper".to_string(),
            ColumnMetadata {
                name: "pl_orbper".to_string(),
                description: Some("Orbital Period".to_string()),
                unit: Some("day".to_string()),
                datatype: "double".to_string(),
            },
        );

        let result = get_columns_metadata(&all_meta, &["pl_name", "pl_orbper"]);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("pl_name").unwrap(), "Planet Name");
        assert!(result.get("pl_orbper").unwrap().contains("Orbital Period"));
        assert!(result.get("pl_orbper").unwrap().contains("[day]"));
    }

    #[test]
    fn test_save_and_load_metadata_toml() {
        use std::env;

        // Create test metadata
        let mut metadata = HashMap::new();
        metadata.insert(
            "hostname".to_string(),
            ColumnMetadata {
                name: "hostname".to_string(),
                description: Some("Host Name".to_string()),
                unit: None,
                datatype: "char".to_string(),
            },
        );
        metadata.insert(
            "dec".to_string(),
            ColumnMetadata {
                name: "dec".to_string(),
                description: None,
                unit: Some("deg".to_string()),
                datatype: "double".to_string(),
            },
        );

        // Create temp file path
        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("test_metadata.toml");

        // Save metadata
        save_metadata_toml(&metadata, &temp_file)
            .expect("Failed to save metadata");

        // Load metadata
        let loaded_metadata =
            load_metadata_toml(&temp_file).expect("Failed to load metadata");

        // Verify
        assert_eq!(loaded_metadata.len(), 2);
        assert_eq!(loaded_metadata.get("hostname").unwrap().name, "hostname");
        assert_eq!(
            loaded_metadata.get("hostname").unwrap().description,
            Some("Host Name".to_string())
        );
        assert_eq!(
            loaded_metadata.get("dec").unwrap().unit,
            Some("deg".to_string())
        );

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
    }
}
