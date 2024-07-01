//! Everything related to "stellarhosts" table,
//! https://exoplanetarchive.ipac.caltech.edu/docs/API_STELLARHOSTS_columns.html
use serde::Serialize;
use votable::iter::{TableIter, VOTableIterator};
use votable::VOTableError;

#[derive(Debug, Serialize)]
struct StellarhostsRecord<'a> {
    hostname: &'a str,
    // hd_name: &'a str,
    // sy_snum: u32,
    // st_age: f64,
    // st_mass: f64,
}

pub fn load_data() -> Result<(), VOTableError> {
    let mut votable_it = VOTableIterator::from_file("data/stellarhosts.vot").unwrap();
    let mut iter_num = 0;
    while let Some(mut row_it) = votable_it.next_table_row_value_iter()? {
        // let table_ref_mut = row_it.table();
        // for elem in table_ref_mut.elems.iter() {
        //     println!("{:?}", elem);
        // }
        // println!("Fields: {:?}", table_ref_mut.elems);
        for (i, row) in row_it.enumerate() {
            // let record = StellarhostsRecord {
            //     hostname: row.unwrap().get(0).unwrap().into(),
            // };
            println!("Row {}: {:?}", i, row);
            iter_num += 1;
            if iter_num > 2 {
                break;
            }
        }
    }
    let votable = votable_it.end_of_it();
    println!("VOTable: {:?}", votable);
    Ok(())
}
