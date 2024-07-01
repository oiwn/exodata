pub mod stellarhosts {
    use serde::{Deserialize, Serialize};
    use surrealdb::engine::local::Mem;
    use surrealdb::sql::Thing;
    use surrealdb::Surreal;
    use tokio;

    #[derive(Debug, Serialize)]
    struct Stellarhost<'a> {
        first: &'a str,
        last: &'a str,
    }

    pub fn load_data() {
        use votable::iter::{TableIter, VOTableIterator};
        let mut votable_it = VOTableIterator::from_file("data/stellarhosts.vot").unwrap();
        let mut iter_num = 0;
        while let Some(mut row_it) = votable_it.next_table_row_value_iter().unwrap() {
            let table_ref_mut = row_it.table();
            println!("Fields: {:?}", table_ref_mut.elems);
            for (i, row) in row_it.enumerate() {
                println!("Row {}: {:?}", i, row);

                iter_num += 1;
                if iter_num > 10 {
                    break;
                }
            }
        }
        let votable = votable_it.end_of_it();
        println!("VOTable: {:?}", votable);
    }

    fn load_into_database() {
        // let rt = tokio::time
    }
}
