use std::collections::{HashMap, HashSet};

use exo_core::metadata::ColumnMetadata;
use votable::iter::{TableIter, VOTableIterator};
use votable::{TableElem, datatype::Datatype, impls::VOTableValue};

/// Print VOTable field headers.
pub fn print_votable_headers(path: &str) {
    let mut votable_it = VOTableIterator::from_file(path).unwrap();
    let mut row = votable_it.next_table_row_value_iter().unwrap().unwrap();
    let table_ref_mut = row.table();
    for elem in table_ref_mut.elems.iter() {
        if let TableElem::Field(field) = elem {
            println!("FIELD. name: {}; datatype: {}", field.name, field.datatype);
        }
    }
}

/// Detect which VOTable columns contain null values.
pub fn detect_nullable_columns(path: &str) -> HashSet<usize> {
    let mut is_null_columns: HashSet<usize> = HashSet::new();
    let mut votable_it = VOTableIterator::from_file(path).unwrap();

    while let Some(row_it) = votable_it.next_table_row_value_iter().unwrap() {
        for (_, row) in row_it.enumerate() {
            match row {
                Ok(r) => {
                    for (i, field) in r.iter().enumerate() {
                        if matches!(field, VOTableValue::Null) {
                            is_null_columns.insert(i);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error getting row! {}", e);
                }
            };
        }
    }

    is_null_columns
}

pub fn extract_coumns_types(path: &str) {
    let mut votable_it =
        VOTableIterator::from_file(path).expect("Failed to read VOTable");
    let mut row = votable_it.next_table_row_value_iter().unwrap().unwrap();
    let header = row.table();
    println!("ROW. {:?}", header);
    for elem in header.elems.iter() {
        println!("{:?}", elem);
    }
}

/// Generates a Rust struct from a VOTable by analyzing all rows for nullability.
pub fn structure_from_votables_codegen(
    path: &str,
    name: &str,
) -> anyhow::Result<()> {
    let mut votable_it =
        VOTableIterator::from_file(path).expect("Failed to read VOTable");

    let mut row = votable_it.next_table_row_value_iter().unwrap().unwrap();
    let header = row.table();

    let mut field_types = vec![];
    let mut nullable_fields = HashSet::new();

    for elem in header.elems.iter() {
        if let TableElem::Field(field) = elem {
            let rust_type = match field.datatype {
                Datatype::CharASCII | Datatype::CharUnicode => "String",
                Datatype::Double => "f64",
                Datatype::Int => "i32",
                Datatype::Float => "f32",
                _ => continue,
            };
            field_types.push((field.name.clone(), rust_type));
        }
    }

    while let Ok(row_result) = votable_it.next_table_row_value_iter() {
        match row_result {
            Some(row_iter) => {
                for (idx, result) in row_iter.enumerate() {
                    match result {
                        Ok(values) => {
                            if values
                                .iter()
                                .any(|v| matches!(v, VOTableValue::Null))
                            {
                                nullable_fields
                                    .insert(field_types[idx].0.clone());
                            }
                        }
                        Err(e) => {
                            eprintln!("Error processing row values: {:?}", e);
                            continue;
                        }
                    }
                }
            }
            None => {
                eprintln!("Error reading row iterator");
                continue;
            }
        }
    }

    let mut structs_code = String::new();
    structs_code
        .push_str("#[derive(Debug, serde::Serialize, serde::Deserialize)]\n");
    structs_code.push_str(format!("struct {} {{\n", name).as_str());

    for (field_name, field_type) in field_types {
        let type_declaration = if nullable_fields.contains(&field_name) {
            format!("Option<{}>", field_type)
        } else {
            field_type.to_string()
        };
        structs_code
            .push_str(&format!("    {}: {},\n", field_name, type_declaration));
    }
    structs_code.push_str("}\n");
    println!("{}", structs_code);
    Ok(())
}

pub fn prev_structure_from_votables_codegen(path: &str, name: &str) {
    let mut votable_it = VOTableIterator::from_file(path).unwrap();

    let mut row = votable_it.next_table_row_value_iter().unwrap().unwrap();
    let table_ref_mut = row.table();

    let mut structs_code = String::new();
    structs_code
        .push_str("#[derive(Debug, serde::Serialize, serde::Deserialize)]\n");
    structs_code.push_str(format!("struct {} {{\n", name).as_str());

    for elem in table_ref_mut.elems.iter() {
        if let TableElem::Field(field) = elem {
            let field_name = &field.name;
            let field_type = match field.datatype {
                Datatype::CharASCII => "Option<String>",
                Datatype::Double => "Option<f64>",
                Datatype::Int => "Option<i32>",
                _ => {
                    panic!("Wrong field!")
                }
            };

            structs_code
                .push_str(&format!("    {}: {},\n", field_name, field_type));
        }
    }
    structs_code.push_str("}\n");
    println!("{}", structs_code);
}

/// Parse VOTable and extract column metadata.
pub fn parse_votable_metadata(
    vot_path: &str,
) -> Result<HashMap<String, ColumnMetadata>, String> {
    let mut votable_it = VOTableIterator::from_file(vot_path)
        .map_err(|e| format!("Failed to parse VOTable: {}", e))?;

    let mut metadata = HashMap::new();

    if let Ok(Some(mut row)) = votable_it.next_table_row_value_iter() {
        let table = row.table();

        for elem in table.elems.iter() {
            if let TableElem::Field(field) = elem {
                let column_metadata = ColumnMetadata {
                    name: field.name.clone(),
                    description: field
                        .description
                        .as_ref()
                        .map(|d| d.to_string()),
                    unit: field.unit.clone(),
                    datatype: format!("{:?}", field.datatype),
                };

                metadata.insert(field.name.clone(), column_metadata);
            }
        }
    }

    Ok(metadata)
}

pub fn get_exoplanets_metadata(
    vot_path: &str,
) -> HashMap<String, ColumnMetadata> {
    parse_votable_metadata(vot_path).unwrap_or_default()
}

pub fn get_stellarhosts_metadata(
    vot_path: &str,
) -> HashMap<String, ColumnMetadata> {
    parse_votable_metadata(vot_path).unwrap_or_default()
}
