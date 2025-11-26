//! Everything related to "stellarhosts" table,
//! https://exoplanetarchive.ipac.caltech.edu/docs/API_STELLARHOSTS_columns.html

use crate::tables::stellarhosts::load_data;
use leptos::prelude::*;
use serde_json;

#[server]
pub async fn get_exoplanet_data() -> Result<String, ServerFnError> {
    match load_data("data/stellarhosts.vot") {
        Ok(df) => {
            // Get column names
            let columns: Vec<String> = df
                .get_column_names_str()
                .iter()
                .map(|&name| name.to_string())
                .collect();

            // Get first few rows as a preview (limit to 10 rows)
            let preview = df.head(Some(10));

            let mut result = Vec::new();

            // Add header row
            result.push(columns);

            // Add data rows
            for i in 0..preview.height() {
                let row_values: Vec<String> = (0..preview.width())
                    .map(|j| {
                        let col_name = preview.get_column_names_str()[j];
                        let col = preview.column(col_name).unwrap();
                        match col.get(i) {
                            Ok(val) => format!("{:?}", val),
                            Err(_) => "null".to_string(),
                        }
                    })
                    .collect();
                result.push(row_values);
            }

            // Convert to JSON string
            match serde_json::to_string(&result) {
                Ok(json) => Ok(json),
                Err(_) => Err(ServerFnError::ServerError(
                    "Failed to serialize data".to_string(),
                )),
            }
        }
        Err(_) => Err(ServerFnError::ServerError(
            "Failed to load data".to_string(),
        )),
    }
}
