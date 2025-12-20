use anyhow::Error;
use polars::prelude::*;

use crate::tables::common::load_parquet;

pub fn load_data(path: &str) -> Result<DataFrame, Error> {
    load_data_with_limit(path, None)
}

pub fn load_data_with_limit(
    path: &str,
    limit: Option<usize>,
) -> Result<DataFrame, Error> {
    load_parquet(path, limit)
}
