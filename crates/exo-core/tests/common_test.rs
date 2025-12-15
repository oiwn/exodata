use anyhow::Error;
use exo_core::tables::common::{
    count_non_null_values, create_histogram, get_numeric_stats, load_parquet,
};

const EXOPLANETS_FIXTURE: &str = "tests/fixtures/exoplanets.fixture";

#[test]
fn test_load_parquet() -> Result<(), Error> {
    let df = load_parquet(EXOPLANETS_FIXTURE, None)?;
    assert_eq!(df.height(), 100, "Should have 100 sampled rows");
    assert!(df.width() > 0, "Should have columns");
    Ok(())
}

#[test]
fn test_load_parquet_with_limit() -> Result<(), Error> {
    let df = load_parquet(EXOPLANETS_FIXTURE, Some(10))?;
    assert_eq!(df.height(), 10, "Should limit to 10 rows");
    Ok(())
}

#[test]
fn test_count_non_null_values() -> Result<(), Error> {
    let df = load_parquet(EXOPLANETS_FIXTURE, None)?;
    let count = count_non_null_values(&df, "pl_name")?;
    assert!(count > 0, "Should have non-null planet names");
    assert!(count <= 100, "Count should not exceed total rows");
    Ok(())
}
#[test]
fn test_get_numeric_stats() -> Result<(), Error> {
    let df = load_parquet(EXOPLANETS_FIXTURE, None)?;

    if let Some(stats) = get_numeric_stats(&df, "pl_masse")? {
        assert!(stats.count > 0);
        assert!(stats.min <= stats.max);
        assert!(stats.mean >= stats.min);
        assert!(stats.mean <= stats.max);
        assert!(stats.std >= 0.0);
    }
    Ok(())
}

#[test]
fn test_create_histogram() -> Result<(), Error> {
    let df = load_parquet(EXOPLANETS_FIXTURE, None)?;
    let histogram = create_histogram(&df, "pl_orbper", 0.0, 1000.0, 10)?;

    assert_eq!(histogram.len(), 10, "Should have 10 bins");

    // Verify bin structure
    for bin in &histogram {
        assert!(bin.min < bin.max, "Bin min should be less than max");
    }
    Ok(())
}
