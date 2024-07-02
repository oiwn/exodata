//! Common functions!
use std::collections::HashSet;
use votable::iter::{TableIter, VOTableIterator};
use votable::{datatype::Datatype, impls::VOTableValue, TableElem};
// use votable::{Field, VOTableElement};

pub struct TableColumn {
    name: String,
    datatype: Datatype,
}

// Print table headers.
pub fn print_votable_headers(path: &str) {
    let mut votable_it = VOTableIterator::from_file(path).unwrap();
    let mut row = votable_it.next_table_row_value_iter().unwrap().unwrap();
    let table_ref_mut = row.table();
    for elem in table_ref_mut.elems.iter() {
        match elem {
            TableElem::Field(field) => {
                println!(
                    "FIELD. name: {}; datatype: {}",
                    field.name, field.datatype
                );
            }
            _ => {}
        }
    }
}

/// Function to detect if columns in a VOTable are nullable or not.
/// Returns a HashMap with field names as keys and a boolean indicating if the field is nullable.
pub fn detect_nullable_columns(path: &str) -> HashSet<usize> {
    let mut is_null_columns: HashSet<usize> = HashSet::new();
    let mut votable_it = VOTableIterator::from_file(path).unwrap();

    while let Some(row_it) = votable_it.next_table_row_value_iter().unwrap() {
        for (_, row) in row_it.enumerate() {
            match row {
                Ok(r) => {
                    // Now iterate over row elements
                    for (i, field) in r.iter().enumerate() {
                        match field {
                            VOTableValue::Null => {
                                is_null_columns.insert(i);
                            }
                            _ => continue,
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
/// Next i supposed to paste it into actual code, where this struct is required
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

    // Extract field names and initial types from the header
    for elem in header.elems.iter() {
        if let TableElem::Field(field) = elem {
            let rust_type = match field.datatype {
                Datatype::CharASCII | Datatype::CharUnicode => "String",
                Datatype::Double => "f64",
                Datatype::Int => "i32",
                Datatype::Float => "f32",
                // Add other necessary mappings
                _ => continue,
            };
            field_types.push((field.name.clone(), rust_type));
        }
    }

    // Scan all rows to determine nullability
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
                continue; // Skip this row and move to the next
            }
        }
    }

    // Generate the struct
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
