use std::time::Duration;

use anyhow::{Error, anyhow};
use polars::prelude::*;
use votable::data::DataElem;
use votable::datatype::Datatype;
use votable::impls::VOTableValue;
use votable::impls::mem::InMemTableDataRows;
use votable::table::TableElem;
use votable::votable::VOTableWrapper;

/// Load a VOTable file into a Polars DataFrame.
/// An optional `limit` allows partial loading for faster inspection.
pub fn load_votable(
    path: &str,
    limit: Option<usize>,
) -> Result<DataFrame, Error> {
    Ok(load_votable_with_progress_timed(path, limit, None)?.0)
}

/// Same as `load_votable` but with a progress bar hook.
pub fn load_votable_with_progress(
    path: &str,
    limit: Option<usize>,
    progress: Option<&indicatif::ProgressBar>,
) -> Result<DataFrame, Error> {
    Ok(load_votable_with_progress_timed(path, limit, progress)?.0)
}

/// Same as `load_votable` but returns timing information and supports progress bars.
pub fn load_votable_with_progress_timed(
    path: &str,
    limit: Option<usize>,
    progress: Option<&indicatif::ProgressBar>,
) -> Result<(DataFrame, LoadTiming), Error> {
    let total_start = std::time::Instant::now();
    let votable_wrapper =
        VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file(path)
            .map_err(|e| anyhow!("Failed to read VOTable file: {}", e))?;
    let votable = votable_wrapper.unwrap();
    let metadata_elapsed = total_start.elapsed();

    let table = votable
        .get_first_table()
        .ok_or_else(|| anyhow!("No table found in VOTable"))?;

    // Get field information.
    let fields: Vec<_> = table
        .elems
        .iter()
        .filter_map(|elem| match elem {
            TableElem::Field(field) => Some(field),
            _ => None,
        })
        .collect();

    let field_names: Vec<String> =
        fields.iter().map(|f| f.name.clone()).collect();

    let mut column_buffers: Vec<ColumnData> = fields
        .iter()
        .map(|field| match field.datatype {
            Datatype::Double => ColumnData::Float64(Vec::new()),
            Datatype::Float => ColumnData::Float32(Vec::new()),
            Datatype::LongInt => ColumnData::Int64(Vec::new()),
            Datatype::Int => ColumnData::Int32(Vec::new()),
            Datatype::ShortInt => ColumnData::Int16(Vec::new()),
            Datatype::Logical => ColumnData::Boolean(Vec::new()),
            Datatype::CharASCII | Datatype::CharUnicode => {
                ColumnData::Text(Vec::new())
            }
            _ => ColumnData::Text(Vec::new()),
        })
        .collect();

    if let Some(data) = &table.data {
        if let DataElem::TableData(table_data) = &data.data {
            let rows_to_process = if let Some(limit) = limit {
                std::cmp::min(limit, table_data.content.rows.len())
            } else {
                table_data.content.rows.len()
            };

            if let Some(pb) = progress {
                pb.set_length(rows_to_process as u64);
                pb.set_position(0);
                pb.set_message("Reading rows");
            }

            let mut processed = 0;
            const CHUNK: usize = 500;
            let row_start = std::time::Instant::now();

            for row in &table_data.content.rows[..rows_to_process] {
                for (i, cell) in row.iter().enumerate() {
                    if i < column_buffers.len() {
                        column_buffers[i].push(cell)?;
                    }
                }
                processed += 1;

                if let Some(pb) = progress {
                    if processed % CHUNK == 0 || processed == rows_to_process {
                        pb.set_position(processed as u64);
                    }
                }
            }

            if let Some(pb) = progress {
                pb.set_position(rows_to_process as u64);
            }

            let rows_elapsed = row_start.elapsed();

            let series_vec: Result<Vec<Series>, Error> = column_buffers
                .into_iter()
                .zip(field_names.iter())
                .map(|(buffer, name)| buffer.to_series(name))
                .collect();

            let series = series_vec?;
            let columns: Vec<Column> =
                series.into_iter().map(Column::from).collect();
            let df = DataFrame::new(columns)
                .map_err(|e| anyhow!("Failed to create DataFrame: {}", e))?;

            return Ok((
                df,
                LoadTiming {
                    metadata: metadata_elapsed,
                    rows: rows_elapsed,
                },
            ));
        }
    }

    Err(anyhow!("No table data found in VOTable"))
}

/// Timing info for VOTable loading stages.
pub struct LoadTiming {
    pub metadata: Duration,
    pub rows: Duration,
}

enum ColumnData {
    Float64(Vec<Option<f64>>),
    Float32(Vec<Option<f32>>),
    Int64(Vec<Option<i64>>),
    Int32(Vec<Option<i32>>),
    Int16(Vec<Option<i16>>),
    Boolean(Vec<Option<bool>>),
    Text(Vec<Option<String>>),
}

impl ColumnData {
    fn push(&mut self, cell: &VOTableValue) -> Result<(), Error> {
        match self {
            ColumnData::Float64(v) => match cell {
                VOTableValue::Double(val) => v.push(Some(*val)),
                VOTableValue::Null => v.push(None),
                _ => v.push(None),
            },
            ColumnData::Float32(v) => match cell {
                VOTableValue::Float(val) => v.push(Some(*val)),
                VOTableValue::Null => v.push(None),
                _ => v.push(None),
            },
            ColumnData::Int64(v) => match cell {
                VOTableValue::Long(val) => v.push(Some(*val)),
                VOTableValue::Null => v.push(None),
                _ => v.push(None),
            },
            ColumnData::Int32(v) => match cell {
                VOTableValue::Int(val) => v.push(Some(*val)),
                VOTableValue::Null => v.push(None),
                _ => v.push(None),
            },
            ColumnData::Int16(v) => match cell {
                VOTableValue::Short(val) => v.push(Some(*val)),
                VOTableValue::Null => v.push(None),
                _ => v.push(None),
            },
            ColumnData::Boolean(v) => match cell {
                VOTableValue::Bool(val) => v.push(Some(*val)),
                VOTableValue::Null => v.push(None),
                _ => v.push(None),
            },
            ColumnData::Text(v) => match cell {
                VOTableValue::CharASCII(c) => v.push(Some(c.to_string())),
                VOTableValue::CharUnicode(c) => v.push(Some(c.to_string())),
                VOTableValue::String(s) => v.push(Some(s.clone())),
                VOTableValue::Null => v.push(None),
                _ => v.push(Some(cell.to_string())),
            },
        }
        Ok(())
    }

    fn to_series(self, name: &str) -> Result<Series, Error> {
        let series = match self {
            ColumnData::Float64(v) => Series::new(name.into(), v),
            ColumnData::Float32(v) => Series::new(name.into(), v),
            ColumnData::Int64(v) => Series::new(name.into(), v),
            ColumnData::Int32(v) => Series::new(name.into(), v),
            ColumnData::Int16(v) => Series::new(name.into(), v),
            ColumnData::Boolean(v) => Series::new(name.into(), v),
            ColumnData::Text(v) => Series::new(name.into(), v),
        };
        Ok(series)
    }
}
