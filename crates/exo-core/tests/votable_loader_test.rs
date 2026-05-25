type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_fixture_loads_correctly() -> TestResult {
    // Verify fixture is valid Parquet
    use exo_core::tables::common::load_parquet;
    let df = load_parquet("tests/fixtures/exoplanets.fixture", None)?;

    assert_eq!(df.height(), 100);
    assert!(df.column("pl_name").is_ok());
    assert!(df.column("discoverymethod").is_ok());

    Ok(())
}
