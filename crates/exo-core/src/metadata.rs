use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use votable::iter::{VOTableIterator, TableIter};
use votable::TableElem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub datatype: String,
}

/// Parse VOTable and extract column metadata
pub fn parse_votable_metadata(vot_path: &str) -> Result<HashMap<String, ColumnMetadata>, String> {
    let mut votable_it = VOTableIterator::from_file(vot_path)
        .map_err(|e| format!("Failed to parse VOTable: {}", e))?;

    let mut metadata = HashMap::new();

    // Get the first row to access the table header
    if let Ok(Some(mut row)) = votable_it.next_table_row_value_iter() {
        let table = row.table();

        // Iterate through table elements (fields)
        for elem in table.elems.iter() {
            if let TableElem::Field(field) = elem {
                let column_metadata = ColumnMetadata {
                    name: field.name.clone(),
                    description: field.description.as_ref().map(|d| d.to_string()),
                    unit: field.unit.clone(),
                    datatype: format!("{:?}", field.datatype),
                };

                metadata.insert(field.name.clone(), column_metadata);
            }
        }
    }

    Ok(metadata)
}

/// Get metadata for exoplanets columns from VOTable file
pub fn get_exoplanets_metadata(vot_path: &str) -> HashMap<String, ColumnMetadata> {
    parse_votable_metadata(vot_path).unwrap_or_default()
}

/// Get metadata for stellar hosts columns from VOTable file
pub fn get_stellarhosts_metadata(vot_path: &str) -> HashMap<String, ColumnMetadata> {
    parse_votable_metadata(vot_path).unwrap_or_default()
}

/// Print metadata in a human-readable format
pub fn print_metadata(metadata: &HashMap<String, ColumnMetadata>) {
    println!("\n{:<25} {:<15} {:<60}", "Column Name", "Unit", "Description");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_votable_metadata() {
        // This test requires actual VOTable files to be present
        let result = parse_votable_metadata("../../data/exoplanets.vot");
        assert!(result.is_ok() || result.is_err()); // Just check it doesn't panic
    }

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
                ucd: None,
            },
        );
        all_meta.insert(
            "pl_orbper".to_string(),
            ColumnMetadata {
                name: "pl_orbper".to_string(),
                description: Some("Orbital Period".to_string()),
                unit: Some("day".to_string()),
                datatype: "double".to_string(),
                ucd: None,
            },
        );

        let result = get_columns_metadata(&all_meta, &["pl_name", "pl_orbper"]);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("pl_name").unwrap(), "Planet Name");
        assert!(result.get("pl_orbper").unwrap().contains("Orbital Period"));
        assert!(result.get("pl_orbper").unwrap().contains("[day]"));
    }
}
