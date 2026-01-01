use anyhow::Error;

#[test]
fn test_exoplanets_loader() -> Result<(), Error> {
    use exo_core::tables::common::{load_data, load_data_with_limit};

    let df = load_data("tests/fixtures/exoplanets.fixture")?;
    assert_eq!(df.height(), 100);

    let limited =
        load_data_with_limit("tests/fixtures/exoplanets.fixture", Some(5))?;
    assert_eq!(limited.height(), 5);

    Ok(())
}

#[test]
fn test_stellarhosts_loader() -> Result<(), Error> {
    use exo_core::tables::common::{load_data, load_data_with_limit};

    let df = load_data("tests/fixtures/stellarhosts.fixture")?;
    assert!(df.height() > 0);

    Ok(())
}
