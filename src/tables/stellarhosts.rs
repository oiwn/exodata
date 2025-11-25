use anyhow::{anyhow, Error};
use leptos::prelude::*;
use polars::prelude::*;
use serde_json;
use votable::data::DataElem;
use votable::datatype::Datatype;
use votable::impls::mem::InMemTableDataRows;
use votable::impls::VOTableValue;
use votable::table::TableElem;
use votable::votable::VOTableWrapper;

pub fn load_data(path: &str) -> Result<DataFrame, Error> {
    let votable_wrapper =
        VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file(path)
            .map_err(|e| anyhow!("Failed to read VOTable file: {}", e))?;
    let votable = votable_wrapper.unwrap();

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

    // Create column buffers.
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
            _ => ColumnData::Text(Vec::new()), // Default for unsupported types
        })
        .collect();

    // Get data rows
    if let Some(data) = &table.data {
        if let DataElem::TableData(table_data) = &data.data {
            for row in &table_data.content.rows {
                for (i, cell) in row.iter().enumerate() {
                    if i < column_buffers.len() {
                        column_buffers[i].push(cell)?;
                    }
                }
            }
        }
    }

    // Convert buffers to Series.
    let series_vec: Result<Vec<Series>, Error> = column_buffers
        .into_iter()
        .zip(field_names.iter())
        .map(|(buffer, name)| buffer.to_series(name))
        .collect();

    let series = series_vec?;
    let columns: Vec<Column> = series.into_iter().map(Column::from).collect();
    DataFrame::new(columns)
        .map_err(|e| anyhow!("Failed to create DataFrame: {}", e))
}

// Enum to hold different types of column data.
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
    // Push a cell value to the correct vector type.
    fn push(&mut self, cell: &VOTableValue) -> Result<(), Error> {
        match self {
            ColumnData::Float64(v) => match cell {
                VOTableValue::Double(val) => v.push(Some(*val)),
                VOTableValue::Null => v.push(None),
                _ => v.push(None), // Or return error
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

    // Convert the buffer to a Polars Series.
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
