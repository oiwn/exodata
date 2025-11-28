use std::process::id;
use std::time::Instant;
use polars::prelude::*;
use votable::data::DataElem;
use votable::datatype::Datatype;
use votable::impls::mem::InMemTableDataRows;
use votable::impls::VOTableValue;
use votable::table::TableElem;
use votable::votable::VOTableWrapper;

/// Simple memory benchmark for exoplanets datasets
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Benchmark for Exoplanets Datasets ===");
    println!();
    
    // Load stellarhosts and measure memory
    println!("Loading stellarhosts data (167MB file)...");
    let start_time = Instant::now();
    let stellarhosts_df = load_data("data/stellarhosts.vot")?;
    let stellarhosts_time = start_time.elapsed();
    let stellarhosts_memory_bytes = stellarhosts_df.estimated_size();
    let stellarhosts_rows = stellarhosts_df.height();
    let stellarhosts_cols = stellarhosts_df.width();
    
    println!("✓ Loaded {} rows, {} columns in {:?}", 
        stellarhosts_rows, 
        stellarhosts_cols, 
        stellarhosts_time);
    println!("Memory usage: {:.2} MB", 
        stellarhosts_memory_bytes as f64 / (1024.0 * 1024.0));
    
    // Load exoplanets and measure memory
    println!("\nLoading exoplanets data (394MB file)...");
    let start_time = Instant::now();
    let exoplanets_df = load_data("data/exoplanets.vot")?;
    let exoplanets_time = start_time.elapsed();
    let exoplanets_memory_bytes = exoplanets_df.estimated_size();
    let exoplanets_rows = exoplanets_df.height();
    let exoplanets_cols = exoplanets_df.width();
    
    println!("✓ Loaded {} rows, {} columns in {:?}", 
        exoplanets_rows, 
        exoplanets_cols, 
        exoplanets_time);
    println!("Memory usage: {:.2} MB", 
        exoplanets_memory_bytes as f64 / (1024.0 * 1024.0));
    
    // Summary report
    println!("\n=== MEMORY USAGE SUMMARY ===");
    
    println!("Dataset        | Rows    | Columns | Memory (MB)");
    println!("---------------|---------|---------|--------------");
    println!("stellarhosts   | {:>8}  | {:>7}     | {:>11.2}", 
        stellarhosts_rows,
        stellarhosts_cols,
        stellarhosts_memory_bytes as f64 / (1024.0 * 1024.0));
    
    println!("exoplanets     | {:>8}  | {:>7}     | {:>11.2}", 
        exoplanets_rows,
        exoplanets_cols,
        exoplanets_memory_bytes as f64 / (1024.0 * 1024.0));
    
    let total_memory_mb = (stellarhosts_memory_bytes + exoplanets_memory_bytes) as f64 / (1024.0 * 1024.0);
    println!("\nCombined Memory Usage: {:.2} MB ({:.2} GB)", 
        total_memory_mb, 
        total_memory_mb / 1024.0);
    
    // File size to memory ratio
    println!("\nFile Size to Memory Ratio:");
    println!("  stellarhosts: 167MB file → {:.2}MB memory ({:.1}x)", 
        stellarhosts_memory_bytes as f64 / (1024.0 * 1024.0),
        stellarhosts_memory_bytes as f64 / (1024.0 * 1024.0) / 167.0);
    
    println!("  exoplanets: 394MB file → {:.2}MB memory ({:.1}x)", 
        exoplanets_memory_bytes as f64 / (1024.0 * 1024.0),
        exoplanets_memory_bytes as f64 / (1024.0 * 1024.0) / 394.0);
    
    // Warnings for high memory usage
    println!();
    if total_memory_mb > 2048.0 { // > 2GB
        println!("⚠️  WARNING: High memory usage ({:.1}GB) - consider using pagination", 
            total_memory_mb / 1024.0);
    }
    
    if exoplanets_memory_bytes > 1024 * 1024 * 1024 { // > 1GB for exoplanets alone
        println!("⚠️  WARNING: Exoplanets dataset uses {:.1}GB - may cause issues on systems with <4GB RAM", 
            exoplanets_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    }
    
    println!("\nPer-row memory efficiency:");
    println!("  stellarhosts: {:.1} KB per row", 
        stellarhosts_memory_bytes as f64 / stellarhosts_rows as f64 / 1024.0);
    println!("  exoplanets: {:.1} KB per row", 
        exoplanets_memory_bytes as f64 / exoplanets_rows as f64 / 1024.0);
    
    Ok(())
}

/// Load data from VOTable into Polars DataFrame
fn load_data(path: &str) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let votable_wrapper =
        VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file(path)?;
    let votable = votable_wrapper.unwrap();

    let table = votable
        .get_first_table()
        .ok_or("No table found in VOTable")?;

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
    let series_vec: Result<Vec<Series>, Box<dyn std::error::Error>> = column_buffers
        .into_iter()
        .zip(field_names.iter())
        .map(|(buffer, name)| buffer.to_series(name))
        .collect();

    let series = series_vec?;
    let columns: Vec<Column> = series.into_iter().map(Column::from).collect();
    DataFrame::new(columns)
        .map_err(|e| e.into())
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
    // Push a cell value to correct vector type.
    fn push(&mut self, cell: &VOTableValue) -> Result<(), Box<dyn std::error::Error>> {
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

    // Convert buffer to a Polars Series.
    fn to_series(self, name: &str) -> Result<Series, Box<dyn std::error::Error>> {
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