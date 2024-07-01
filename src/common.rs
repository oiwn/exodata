//! Common functions!
use votable::datatype::Datatype;
use votable::iter::{TableIter, VOTableIterator};
use votable::TableElem;

// Print table headers.
pub fn print_votable_headers(path: &str) {
    let mut votable_it = VOTableIterator::from_file(path).unwrap();
    let mut row = votable_it.next_table_row_value_iter().unwrap().unwrap();
    let table_ref_mut = row.table();
    for elem in table_ref_mut.elems.iter() {
        match elem {
            TableElem::Field(field) => {
                println!("FIELD. name: {}; datatype: {}", field.name, field.datatype);
            }
            _ => {}
        }
    }
}

// Code gen to build structure
pub fn print_structure_from(path: &str, name: &str) {
    let mut votable_it = VOTableIterator::from_file(path).unwrap();
    let mut row = votable_it.next_table_row_value_iter().unwrap().unwrap();
    let table_ref_mut = row.table();

    let mut structs_code = String::new();
    structs_code.push_str("#[derive(Debug, serde::Serialize, serde::Deserialize)]\n");
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

            structs_code.push_str(&format!("    {}: {},\n", field_name, field_type));
        }
    }
    structs_code.push_str("}\n");
    println!("{}", structs_code);
}
