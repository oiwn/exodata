//! Offline, deterministic evidence preparation for prose experiments.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, bail};
use exo_core::selection::{exoplanet_record_index, stellar_host_record_index};
use polars::prelude::{ChunkCompareEq, DataFrame, ParquetReader, SerReader};
use serde::Serialize;
use serde_json::{Value, json};

const GUIDE: &str =
    include_str!("../../../../content/prompts/stellarhost_guide.toml");
const LIGHT_YEARS_PER_PARSEC: f64 = 3.26156;
const EARTH_DAYS_PER_YEAR: f64 = 365.0;
const EARTH_MASSES_PER_JUPITER: f64 = 317.8;
const SUN_EFFECTIVE_TEMPERATURE_K: f64 = 5772.0;

#[derive(Serialize)]
struct Measurement {
    source_field: &'static str,
    value: f64,
    unit: &'static str,
    qualifier: &'static str,
    display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_plus: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_minus: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<String>,
}

impl Measurement {
    fn interval(&self) -> Option<(f64, f64)> {
        (self.qualifier == "estimate").then(|| {
            (
                self.value - self.error_minus.unwrap_or(0.0).abs(),
                self.value + self.error_plus.unwrap_or(0.0).abs(),
            )
        })
    }

    fn scale(&mut self, factor: f64, unit: &'static str) {
        self.value *= factor;
        self.error_plus = self.error_plus.map(|v| v * factor);
        self.error_minus = self.error_minus.map(|v| v * factor);
        self.unit = unit;
        self.display = display(self.value, unit, self.qualifier);
    }
}

#[derive(Serialize)]
struct Planet {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    discovery_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discovery_year: Option<i64>,
    measurements: BTreeMap<&'static str, Measurement>,
}

#[derive(Serialize)]
struct System {
    hostname: String,
    matching_host_planet_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_system_planet_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_system_star_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distance: Option<Measurement>,
}

#[derive(Serialize)]
struct Star {
    #[serde(skip_serializing_if = "Option::is_none")]
    spectral_type: Option<String>,
    measurements: BTreeMap<&'static str, Measurement>,
}

#[derive(Serialize)]
struct Request {
    schema_version: u32,
    request: BTreeMap<&'static str, &'static str>,
    guide: BTreeMap<String, String>,
    source: Value,
    system: System,
    star: Star,
    planets: Vec<Planet>,
    publishable_comparisons: Vec<String>,
    silent_constraints: Vec<String>,
}

pub fn columns() -> Vec<String> {
    ["hostname", "evidence_path", "request_path", "diagnostics"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub fn system_id(hostname: &str) -> Result<String> {
    let mut result = String::new();
    for c in hostname.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        } else if (c.is_whitespace() || c == '-') && !result.ends_with('-') {
            result.push('-');
        }
    }
    let result = result.trim_matches('-').to_owned();
    if result.is_empty() {
        bail!("Hostname produces an empty system identifier");
    }
    Ok(result)
}

fn selected_rows(frame: &DataFrame, hostname: &str) -> Result<Vec<Value>> {
    let mask = frame.column("hostname")?.str()?.equal(hostname);
    crate::output::dataframe_to_json(&frame.filter(&mask)?)
}

pub fn run(
    data_dir: &Path,
    output_dir: &Path,
    hostname: &str,
    force: bool,
) -> Result<Value> {
    let id = system_id(hostname)?;
    let host_path = data_dir.join("stellarhosts.parquet");
    let planet_path = data_dir.join("exoplanets.parquet");
    let load = |path: &Path| -> Result<DataFrame> {
        ParquetReader::new(fs::File::open(path)?)
            .finish()
            .with_context(|| format!("Cannot read {}", path.display()))
    };
    let hosts = load(&host_path)?;
    let planets = load(&planet_path)?;
    for frame in [&hosts, &planets] {
        let column = frame.column("hostname")?.str()?;
        let names: BTreeSet<_> =
            (0..column.len()).filter_map(|i| column.get(i)).collect();
        for other in names {
            if other != hostname && system_id(other).ok().as_deref() == Some(&id)
            {
                bail!(
                    "System identifier {id:?} collides for {hostname:?} and {other:?}"
                );
            }
        }
    }
    let host_rows = selected_rows(&hosts, hostname)?;
    let planet_rows = selected_rows(&planets, hostname)?;
    let (mut request, mut evidence, diagnostics) =
        prepare(hostname, &host_rows, &planet_rows)?;
    let paths = json!({"stellarhosts": host_path, "exoplanets": planet_path});
    request.source["files"] = paths.clone();
    evidence["files"] = paths;
    let request_text = toml::to_string_pretty(&request)?;
    let evidence_text = serde_json::to_string_pretty(&evidence)? + "\n";
    let directory = output_dir.join(id);
    fs::create_dir_all(&directory)?;
    write_pair(
        &directory,
        hostname,
        force,
        &evidence_text,
        &request_text,
        |from, to| fs::rename(from, to),
    )?;
    Ok(
        json!({"hostname": hostname, "evidence_path": directory.join("evidence.json"),
        "request_path": directory.join("request.toml"), "diagnostics": diagnostics.join("; ")}),
    )
}

// The installer argument permits deterministic failure injection in tests.
fn write_pair(
    directory: &Path,
    hostname: &str,
    force: bool,
    evidence: &str,
    request: &str,
    install: impl FnMut(&Path, &Path) -> std::io::Result<()>,
) -> Result<()> {
    store_pair(
        directory,
        ".prepare",
        [("evidence.json", evidence), ("request.toml", request)],
        || check_existing(directory, hostname, force),
        install,
    )
}

pub(super) fn store_pair(
    directory: &Path,
    staging_name: &str,
    files: [(&str, &str); 2],
    validate: impl FnOnce() -> Result<()>,
    mut install: impl FnMut(&Path, &Path) -> std::io::Result<()>,
) -> Result<()> {
    let staging = directory.join(staging_name);
    fs::create_dir(&staging).with_context(|| {
        format!(
            "Cannot create {}; another preparation or recovery may be pending",
            staging.display()
        )
    })?;
    let mut recovery_failed = false;
    let result = (|| -> Result<()> {
        validate()?;
        let mut had_previous = [false; 2];
        for (i, (name, text)) in files.iter().enumerate() {
            fs::write(staging.join(name), text)
                .with_context(|| format!("Cannot stage {name}"))?;
            let previous = directory.join(name);
            had_previous[i] = previous.try_exists()?;
            if had_previous[i] {
                fs::copy(&previous, staging.join(format!("{name}.backup")))
                    .with_context(|| format!("Cannot back up {name}"))?;
            }
        }
        for (i, (name, _)) in files.iter().enumerate() {
            if let Err(error) =
                install(&staging.join(name), &directory.join(name))
            {
                let mut failures = Vec::new();
                for j in (0..i).rev() {
                    let name = files[j].0;
                    let restored = if had_previous[j] {
                        fs::rename(
                            staging.join(format!("{name}.backup")),
                            directory.join(name),
                        )
                    } else {
                        fs::remove_file(directory.join(name))
                    };
                    if let Err(error) = restored {
                        failures.push(format!("{name}: {error}"));
                    }
                }
                if !failures.is_empty() {
                    recovery_failed = true;
                    bail!(
                        "Cannot install {name}: {error}; rollback failed ({}); recovery files retained in {}",
                        failures.join("; "),
                        staging.display()
                    );
                }
                bail!(
                    "Cannot install {name}: {error}; previous preparation pair restored"
                );
            }
        }
        Ok(())
    })();
    if !recovery_failed {
        if let Err(error) = fs::remove_dir_all(&staging) {
            return match result {
                Ok(()) => Err(error).context(
                    "Preparation pair installed, but staging cleanup failed",
                ),
                Err(previous) => Err(previous
                    .context(format!("Staging cleanup also failed: {error}"))),
            };
        }
    }
    result
}

fn check_existing(directory: &Path, hostname: &str, force: bool) -> Result<()> {
    for name in ["evidence.json", "request.toml", "metadata.toml"] {
        let path = directory.join(name);
        if !path.try_exists()? {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        let stored = if name == "evidence.json" {
            serde_json::from_str::<Value>(&text)?
        } else {
            serde_json::to_value(toml::from_str::<toml::Value>(&text)?)?
        };
        let identity = if name == "request.toml" {
            &stored["system"]["hostname"]
        } else {
            &stored["hostname"]
        };
        if identity.as_str() != Some(hostname) {
            bail!("Stored hostname mismatch in {}", path.display());
        }
        if name != "metadata.toml" && !force {
            bail!(
                "{} already exists; use --force to replace preparation files",
                path.display()
            );
        }
    }
    Ok(())
}

fn string(row: &Value, field: &str) -> Option<String> {
    row[field]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
}

fn number(row: &Value, field: &str) -> Option<f64> {
    row[field].as_f64().filter(|v| v.is_finite())
}

fn significant(value: f64) -> String {
    if value == 0.0 {
        return "0".into();
    }
    let places = 2 - value.abs().log10().floor() as i32;
    if !(-6..=8).contains(&places) {
        return format!("{value:.2e}");
    }
    let factor = 10f64.powi(places);
    let rounded = (value * factor).round() / factor;
    let text = format!("{:.*}", places.max(0) as usize, rounded);
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    }
}

fn display(value: f64, unit: &str, qualifier: &str) -> String {
    match qualifier {
        "upper_limit" => format!("a reported upper limit of {value} {unit}"),
        "lower_limit" => format!("a reported lower limit of {value} {unit}"),
        "estimate" => format!("about {} {unit}", significant(value)),
        _ => format!(
            "a reported value of {} {unit} (qualifier unavailable)",
            significant(value)
        ),
    }
}

fn measurement(
    row: &Value,
    field: &'static str,
    unit: &'static str,
    name: &str,
    diagnostics: &mut Vec<String>,
) -> Option<Measurement> {
    let Some(value) = number(row, field).filter(|v| *v > 0.0) else {
        diagnostics.push(format!("{name}: {field} missing or unusable"));
        return None;
    };
    let qualifier = match row[format!("{field}lim")].as_i64() {
        Some(0) => "estimate",
        Some(1) => "upper_limit",
        Some(-1) => "lower_limit",
        // The stellarhosts distance column has errors but no limit companion.
        None if field == "sy_dist" => "estimate",
        _ => {
            diagnostics
                .push(format!("{name}: {field} limit qualifier unavailable"));
            "unspecified"
        }
    };
    Some(Measurement {
        source_field: field,
        value,
        unit,
        qualifier,
        display: display(value, unit, qualifier),
        error_plus: number(row, &format!("{field}err1")),
        error_minus: number(row, &format!("{field}err2")),
        provenance: None,
    })
}

fn prepare(
    hostname: &str,
    host_rows: &[Value],
    planet_rows: &[Value],
) -> Result<(Request, Value, Vec<String>)> {
    let host = &host_rows[stellar_host_record_index(host_rows)
        .context("No usable stellar-host summary row")?];
    let mut groups = BTreeMap::<&str, Vec<Value>>::new();
    for row in planet_rows {
        let name = row["pl_name"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .context("Planet row has no usable pl_name")?;
        groups.entry(name).or_default().push(row.clone());
    }
    if groups.is_empty() {
        bail!("No planet rows match hostname {hostname:?}");
    }
    let mut diagnostics = Vec::new();
    let mut selected = Vec::new();
    let mut planets = Vec::new();
    for (name, rows) in groups {
        let row = &rows[exoplanet_record_index(&rows).with_context(|| {
            format!("{name}: expected one unique default row")
        })?];
        selected.push(
            json!({"name": name, "source_row_count": rows.len(), "row": row}),
        );
        let mut measurements = BTreeMap::new();
        for (key, field, unit) in [
            ("orbital_period", "pl_orbper", "days"),
            ("radius", "pl_rade", "Earth radii"),
        ] {
            if let Some(m) = measurement(row, field, unit, name, &mut diagnostics)
            {
                measurements.insert(key, m);
            }
        }
        let mass_field = if row["pl_bmasse"].is_null() {
            "pl_masse"
        } else {
            "pl_bmasse"
        };
        if let Some(mut mass) =
            measurement(row, mass_field, "Earth masses", name, &mut diagnostics)
        {
            let provenance = if mass_field == "pl_masse" {
                Some("Mass".to_owned())
            } else {
                string(row, "pl_bmassprov")
            };
            mass.display = match provenance.as_deref() {
                Some("Msini") => {
                    format!("minimum-mass quantity (M sin i): {}", mass.display)
                }
                Some("Mass") => format!(
                    "planet mass (not a minimum-mass quantity): {}",
                    mass.display
                ),
                Some("M-R relationship") => format!(
                    "mass inferred from a mass-radius relationship: {}",
                    mass.display
                ),
                _ => {
                    diagnostics.push(format!(
                        "{name}: mass provenance unavailable or unrecognized"
                    ));
                    format!(
                        "mass quantity with unspecified provenance: {}",
                        mass.display
                    )
                }
            };
            mass.provenance = provenance;
            measurements.insert("mass", mass);
        }
        planets.push(Planet {
            name: name.to_owned(),
            discovery_method: string(row, "discoverymethod"),
            discovery_year: row["disc_year"].as_i64(),
            measurements,
        });
    }
    let mut star = Star {
        spectral_type: string(host, "st_spectype"),
        measurements: BTreeMap::new(),
    };
    if star.spectral_type.is_none() {
        if let Some(spectype) = host_rows
            .iter()
            .filter_map(|row| string(row, "st_spectype"))
            .next()
        {
            diagnostics.push(
                "st_spectype missing from the selected host row; taken from another host row"
                    .into(),
            );
            star.spectral_type = Some(spectype);
        }
    }
    for (key, field, unit) in [
        ("temperature", "st_teff", "K"),
        ("mass", "st_mass", "solar masses"),
        ("radius", "st_rad", "solar radii"),
        ("age", "st_age", "billion years"),
    ] {
        if let Some(m) =
            measurement(host, field, unit, hostname, &mut diagnostics)
        {
            star.measurements.insert(key, m);
        }
    }
    let mut distance =
        measurement(host, "sy_dist", "parsecs", hostname, &mut diagnostics);
    if let Some(m) = distance.as_mut() {
        m.scale(LIGHT_YEARS_PER_PARSEC, "light-years");
    }
    let catalog_count = host["sy_pnum"].as_i64();
    if catalog_count.is_some_and(|n| n != planets.len() as i64) {
        diagnostics.push("Catalog system planet count differs from exact-host matching planet count; do not assign additional planets to this host".into());
    }
    let system = System {
        hostname: hostname.to_owned(),
        matching_host_planet_count: planets.len(),
        catalog_system_planet_count: catalog_count,
        catalog_system_star_count: host["sy_snum"].as_i64(),
        distance,
    };
    let comparisons = comparisons(&star, &planets);
    let source = json!({"evidence_file": "evidence.json", "distance_conversion": "parsecs * 3.26156 = light-years",
        "selection": "Fullest stellar-host summary row and unique default planet rows; spectral type may be taken from any host row"});
    let evidence = json!({"schema_version": 1, "hostname": hostname,
        "host": {"source_row_count": host_rows.len(), "row": host}, "planets": selected,
        "diagnostics": diagnostics});
    let mut guide: BTreeMap<String, String> = toml::from_str(GUIDE)?;
    if !planets.iter().any(|p| {
        p.measurements
            .get("mass")
            .is_some_and(|m| m.provenance.as_deref() == Some("Msini"))
    }) {
        guide.remove("minimum_mass");
    }
    if !planets
        .iter()
        .any(|p| p.discovery_method.as_deref() == Some("Transit"))
    {
        guide.remove("transit");
    }
    if star.spectral_type.is_none() {
        guide.remove("spectral");
    }
    let request = Request { schema_version: 1,
        request: BTreeMap::from([
            ("task", "Describe this stellar host and its associated planets for curious general readers."),
            ("target_words", "300–600; shorter when needed to avoid repetition or unsupported claims"),
            ("format", "A factual Markdown title and connected paragraphs; no tables or bullet lists"),
            ("coverage", "Introduce the host, name its associated planets, and select useful supplied measurements. Preserve all qualifiers. Do not recite every number or force a generic ending."),
        ]),
        guide, source, system, star, planets,
        publishable_comparisons: comparisons,
        silent_constraints: vec![
            "Only system, star, planets, publishable_comparisons, and guide supply article content; source is audit context.".into(),
            "These constraints and diagnostics are silent instructions, never reader-facing prose.".into(),
            "Missing measurements are unknown. Do not infer composition, density, habitability, orbital spacing, orbital speed, or observing feasibility.".into(),
            "Use only approved comparisons. Do not calculate new ratios or rank masses; preserve minimum-mass provenance and upper/lower limits.".into(),
            "Catalog system counts can include other hosts. Describe only the planets explicitly associated with this hostname; name order is not orbital order.".into(),
        ],
    };
    Ok((request, evidence, diagnostics))
}

fn against(m: &Measurement, reference: f64) -> Option<&'static str> {
    let (low, high) = m.interval()?;
    if significant(m.value) == significant(reference) {
        return None;
    }
    if high < reference {
        Some("smaller")
    } else if low > reference {
        Some("larger")
    } else {
        None
    }
}

fn distinct_from_all(values: &[(&str, &Measurement)], index: usize) -> bool {
    let digits = significant(values[index].1.value);
    values
        .iter()
        .enumerate()
        .all(|(i, (_, m))| i == index || significant(m.value) != digits)
}

fn mass_noun(m: &Measurement) -> &'static str {
    if m.provenance.as_deref() == Some("Msini") {
        "minimum-mass quantity"
    } else {
        "mass estimate"
    }
}

fn comparisons(star: &Star, planets: &[Planet]) -> Vec<String> {
    let mut result = Vec::new();
    for key in ["mass", "radius"] {
        if let Some(direction) =
            star.measurements.get(key).and_then(|m| against(m, 1.0))
        {
            result.push(format!("The star's reported {key} estimate is {direction} than the Sun's."));
        }
    }
    if let Some(direction) = star
        .measurements
        .get("temperature")
        .and_then(|m| against(m, SUN_EFFECTIVE_TEMPERATURE_K))
    {
        let direction = if direction == "larger" {
            "hotter"
        } else {
            "cooler"
        };
        result.push(format!(
            "The star's reported temperature is {direction} than the Sun's."
        ));
    }
    for planet in planets {
        if let Some(direction) = planet
            .measurements
            .get("radius")
            .and_then(|m| against(m, 1.0))
        {
            result.push(format!(
                "{}'s reported radius estimate is {direction} than Earth's.",
                planet.name
            ));
        }
        if let Some(m) = planet.measurements.get("mass") {
            if let Some(direction) = against(m, 1.0) {
                result.push(format!(
                    "{}'s reported {} is {direction} than Earth's.",
                    planet.name,
                    mass_noun(m)
                ));
            }
            if against(m, EARTH_MASSES_PER_JUPITER) == Some("larger") {
                result.push(format!(
                    "{}'s reported {} is roughly {} times Jupiter's mass.",
                    planet.name,
                    mass_noun(m),
                    significant(m.value / EARTH_MASSES_PER_JUPITER)
                ));
            }
        }
        if let Some(direction) = planet
            .measurements
            .get("orbital_period")
            .and_then(|m| against(m, EARTH_DAYS_PER_YEAR))
        {
            let direction = if direction == "smaller" {
                "shorter"
            } else {
                "longer"
            };
            result.push(format!(
                "{}'s year is {direction} than Earth's roughly 365-day year.",
                planet.name
            ));
        }
    }
    if planets.len() < 2 {
        return result;
    }
    for (key, low_label, high_label) in [
        ("orbital_period", "shortest year", "longest year"),
        ("radius", "smallest radius", "largest radius"),
    ] {
        let mut values: Vec<_> = planets
            .iter()
            .filter_map(|p| {
                p.measurements
                    .get(key)
                    .filter(|m| m.interval().is_some())
                    .map(|m| (p.name.as_str(), m))
            })
            .collect();
        if values.len() != planets.len() {
            continue;
        }
        values.sort_by(|a, b| a.1.value.total_cmp(&b.1.value));
        let last = values.len() - 1;
        if distinct_from_all(&values, 0) {
            result.push(format!("Among the listed planets, {} has the {low_label} by reported estimates ({}).", values[0].0, values[0].1.display));
        }
        if distinct_from_all(&values, last) {
            result.push(format!("Among the listed planets, {} has the {high_label} by reported estimates ({}).", values[last].0, values[last].1.display));
        }
        if values.len() >= 3 {
            if distinct_from_all(&values, 1) {
                result.push(format!("Among the listed planets, {} has the second-{low_label} by reported estimates ({}).", values[1].0, values[1].1.display));
            }
            if distinct_from_all(&values, last - 1) {
                result.push(format!("Among the listed planets, {} has the second-{high_label} by reported estimates ({}).", values[last - 1].0, values[last - 1].1.display));
            }
        }
    }
    let mut masses: Vec<_> = planets
        .iter()
        .filter_map(|p| {
            p.measurements
                .get("mass")
                .filter(|m| m.interval().is_some())
                .map(|m| (p.name.as_str(), m))
        })
        .collect();
    if masses.len() == planets.len() {
        let provenance = masses[0].1.provenance.as_deref();
        if matches!(provenance, Some("Mass") | Some("Msini"))
            && masses
                .iter()
                .all(|(_, m)| m.provenance.as_deref() == provenance)
        {
            let label = if provenance == Some("Msini") {
                "reported minimum-mass quantity"
            } else {
                "reported mass"
            };
            masses.sort_by(|a, b| a.1.value.total_cmp(&b.1.value));
            let last = masses.len() - 1;
            let amount = |index: usize| {
                format!(
                    "about {} {}",
                    significant(masses[index].1.value),
                    masses[index].1.unit
                )
            };
            if distinct_from_all(&masses, 0) {
                result.push(format!(
                    "Among the listed planets, {} has the smallest {label} ({}).",
                    masses[0].0,
                    amount(0)
                ));
            }
            if distinct_from_all(&masses, last) {
                result.push(format!(
                    "Among the listed planets, {} has the largest {label} ({}).",
                    masses[last].0,
                    amount(last)
                ));
            }
            if masses.len() >= 3 {
                if distinct_from_all(&masses, 1) {
                    result.push(format!(
                        "Among the listed planets, {} has the second-smallest {label} ({}).",
                        masses[1].0,
                        amount(1)
                    ));
                }
                if distinct_from_all(&masses, last - 1) {
                    result.push(format!(
                        "Among the listed planets, {} has the second-largest {label} ({}).",
                        masses[last - 1].0,
                        amount(last - 1)
                    ));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(std::path::PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "exodata-pair-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn pair_installation_succeeds_or_restores_previous_files_and_absences() {
        let old = [
            r#"{"hostname":"Test","revision":1}"#,
            "[system]\nhostname = 'Test'\nrevision = 1\n",
        ];
        let new = [
            r#"{"hostname":"Test","revision":2}"#,
            "[system]\nhostname = 'Test'\nrevision = 2\n",
        ];
        let names = ["evidence.json", "request.toml"];
        for previous in
            [[true, true], [false, false], [false, true], [true, false]]
        {
            for fail_at in [None, Some(0), Some(1)] {
                let directory = TestDirectory::new();
                for i in 0..2 {
                    if previous[i] {
                        fs::write(directory.0.join(names[i]), old[i]).unwrap();
                    }
                }
                fs::write(directory.0.join("description.md"), "Manual article")
                    .unwrap();
                fs::write(
                    directory.0.join("metadata.toml"),
                    "hostname = 'Test'\n",
                )
                .unwrap();
                let mut call = 0;
                let result = write_pair(
                    &directory.0,
                    "Test",
                    true,
                    new[0],
                    new[1],
                    |from, to| {
                        let index = call;
                        call += 1;
                        if fail_at == Some(index) {
                            Err(std::io::Error::other(
                                "simulated installation failure",
                            ))
                        } else {
                            fs::rename(from, to)
                        }
                    },
                );
                assert_eq!(result.is_ok(), fail_at.is_none());
                if let Err(error) = result {
                    assert!(
                        error
                            .to_string()
                            .contains("previous preparation pair restored")
                    );
                }
                for i in 0..2 {
                    let actual =
                        fs::read_to_string(directory.0.join(names[i])).ok();
                    let expected = if fail_at.is_none() {
                        Some(new[i])
                    } else if previous[i] {
                        Some(old[i])
                    } else {
                        None
                    };
                    assert_eq!(actual.as_deref(), expected);
                }
                assert!(!directory.0.join(".prepare").exists());
                assert_eq!(
                    fs::read_to_string(directory.0.join("description.md"))
                        .unwrap(),
                    "Manual article"
                );
                assert_eq!(
                    fs::read_to_string(directory.0.join("metadata.toml"))
                        .unwrap(),
                    "hostname = 'Test'\n"
                );
            }
        }
    }

    #[test]
    fn pending_recovery_blocks_preparation_without_deleting_backups() {
        let directory = TestDirectory::new();
        let staging = directory.0.join(".prepare");
        fs::create_dir(&staging).unwrap();
        fs::write(staging.join("evidence.json.backup"), "Recovery evidence")
            .unwrap();
        let result = write_pair(
            &directory.0,
            "Test",
            true,
            "new evidence",
            "new request",
            |_, _| panic!("must not install while recovery is pending"),
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("recovery may be pending")
        );
        assert_eq!(
            fs::read_to_string(staging.join("evidence.json.backup")).unwrap(),
            "Recovery evidence"
        );
        assert!(!directory.0.join("evidence.json").exists());
        assert!(!directory.0.join("request.toml").exists());
    }

    fn host() -> Value {
        json!({"hostname": "Test", "st_mass": 1.0, "st_masslim": 0,
            "st_age": 5.0, "st_agelim": -1, "sy_dist": 10.0,
            "sy_pnum": 2, "sy_snum": 1, "st_refname": "Selected"})
    }

    fn planet(name: &str, period: f64) -> Value {
        json!({"hostname": "Test", "pl_name": name, "default_flag": 1,
            "pl_orbper": period, "pl_orbperlim": 0, "pl_orbpererr1": 0.01, "pl_orbpererr2": -0.01,
            "pl_rade": null, "pl_bmasse": 25.0, "pl_bmasselim": 1, "pl_bmassprov": "Mass"})
    }

    #[test]
    fn selection_preserves_gaps_bounds_and_exact_source_rows() {
        let h = host();
        let mut other = planet("Test b", 9.0);
        other["default_flag"] = json!(0);
        other["pl_rade"] = json!(3.0);
        let selected = planet("Test b", 10.0);
        let rows = vec![other, selected.clone()];
        let (request, evidence, diagnostics) =
            prepare("Test", &[h.clone()], &rows).unwrap();
        assert_eq!(evidence["host"]["row"], h);
        assert_eq!(evidence["planets"][0]["row"], selected);
        assert!(!request.planets[0].measurements.contains_key("radius"));
        assert_eq!(
            request.planets[0].measurements["mass"].qualifier,
            "upper_limit"
        );
        assert_eq!(
            request.star.measurements["age"].display,
            "a reported lower limit of 5 billion years"
        );
        assert!(diagnostics.iter().any(|s| s.contains("count differs")));
        assert_eq!(request.system.matching_host_planet_count, 1);
        assert_eq!(request.system.catalog_system_planet_count, Some(2));
        let distance = request.system.distance.unwrap();
        assert!((distance.value - 32.6156).abs() < 1e-10);
        assert_eq!(distance.display, "about 32.6 light-years");
    }

    #[test]
    fn rejects_missing_or_ambiguous_selection() {
        let p = planet("Test b", 10.0);
        assert!(prepare("Test", &[], &[p.clone()]).is_err());
        assert!(prepare("Test", &[host()], &[]).is_err());
        assert!(prepare("Test", &[host()], &[p.clone(), p.clone()]).is_err());
        let mut nondefault = p;
        nondefault["default_flag"] = json!(0);
        assert!(prepare("Test", &[host()], &[nondefault]).is_err());
    }

    #[test]
    fn minimum_mass_and_same_row_fallback_retain_qualifiers() {
        let mut p = planet("Test b", 10.0);
        p["pl_bmassprov"] = json!("Msini");
        let (request, _, _) = prepare("Test", &[host()], &[p.clone()]).unwrap();
        let m = &request.planets[0].measurements["mass"];
        assert_eq!(m.provenance.as_deref(), Some("Msini"));
        assert!(request.guide.contains_key("minimum_mass"));
        assert!(m.display.contains("minimum-mass quantity"));
        assert!(m.display.contains("upper limit"));
        p["pl_bmasse"] = Value::Null;
        p["pl_masse"] = json!(4.56789);
        p["pl_masselim"] = json!(-1);
        let (request, _, _) = prepare("Test", &[host()], &[p]).unwrap();
        let m = &request.planets[0].measurements["mass"];
        assert_eq!(m.source_field, "pl_masse");
        assert_eq!(m.provenance.as_deref(), Some("Mass"));
        assert!(!request.guide.contains_key("minimum_mass"));
        assert!(!request.guide.contains_key("transit"));
        assert!(
            !request
                .silent_constraints
                .iter()
                .any(|s| s.contains("missing or unusable"))
        );
        assert!(m.display.contains("lower limit of 4.56789"));
    }

    #[test]
    fn spectral_type_filled_from_any_host_row_and_guide_follows() {
        let other = json!({"hostname": "Test", "st_spectype": "K1/K2 V",
            "st_refname": "Other"});
        let (request, evidence, diagnostics) =
            prepare("Test", &[host(), other], &[planet("Test b", 10.0)]).unwrap();
        assert_eq!(request.star.spectral_type.as_deref(), Some("K1/K2 V"));
        assert!(request.guide.contains_key("spectral"));
        assert!(
            diagnostics
                .iter()
                .any(|s| s.contains("taken from another host row"))
        );
        assert!(evidence["host"]["row"]["st_spectype"].is_null());
        let (request, _, _) =
            prepare("Test", &[host()], &[planet("Test b", 10.0)]).unwrap();
        assert!(request.star.spectral_type.is_none());
        assert!(!request.guide.contains_key("spectral"));
        assert!(
            !request
                .silent_constraints
                .iter()
                .any(|s| s.contains("spectral"))
        );
    }

    #[test]
    fn suppresses_overlaps_bounds_unknown_qualifiers_and_rounded_ties() {
        let mut diagnostics = Vec::new();
        let a = measurement(
            &planet("a", 10.0),
            "pl_orbper",
            "days",
            "a",
            &mut diagnostics,
        )
        .unwrap();
        let mut bound = measurement(
            &planet("b", 20.0),
            "pl_orbper",
            "days",
            "b",
            &mut diagnostics,
        )
        .unwrap();
        bound.qualifier = "upper_limit";
        assert_eq!(against(&a, 365.0), Some("smaller"));
        assert!(against(&bound, 365.0).is_none());
        bound.qualifier = "unspecified";
        assert!(against(&bound, 365.0).is_none());
        let mut tie = measurement(
            &planet("c", 19.98),
            "pl_orbper",
            "days",
            "c",
            &mut diagnostics,
        )
        .unwrap();
        tie.value = 10.0;
        let values: Vec<(&str, &Measurement)> =
            [("a", &a), ("c", &tie)].into_iter().collect();
        assert!(!distinct_from_all(&values, 0));
        assert!(!distinct_from_all(&values, 1));
    }

    #[test]
    fn mass_and_temperature_comparisons_follow_provenance() {
        let mut giant = planet("Test b", 10.0);
        giant["pl_bmasse"] = json!(2000.0);
        giant["pl_bmasselim"] = json!(0);
        giant["pl_bmasseerr1"] = json!(100.0);
        giant["pl_bmasseerr2"] = json!(-100.0);
        let mut small = planet("Test c", 20.0);
        small["pl_bmasse"] = json!(4.5);
        small["pl_bmasselim"] = json!(0);
        small["pl_bmasseerr1"] = json!(0.2);
        small["pl_bmasseerr2"] = json!(-0.2);
        let mut h = host();
        h["st_teff"] = json!(3096.0);
        h["st_tefflim"] = json!(0);
        let (request, _, _) =
            prepare("Test", &[h.clone()], &[giant.clone(), small.clone()])
                .unwrap();
        let facts = &request.publishable_comparisons;
        assert!(
            facts.iter().any(|s| s
                == "The star's reported temperature is cooler than the Sun's.")
        );
        assert!(
            facts.iter().any(|s| s
                == "Test b's reported mass estimate is larger than Earth's.")
        );
        assert!(facts.iter().any(|s| s.contains(
            "Test b's reported mass estimate is roughly 6.29 times Jupiter's mass."
        )));
        assert!(
            facts.iter().any(|s| s
                == "Test c's reported mass estimate is larger than Earth's.")
        );
        assert!(
            !facts
                .iter()
                .any(|s| s.contains("Test c") && s.contains("Jupiter"))
        );
        assert!(facts.iter().any(|s| s.contains(
            "Among the listed planets, Test b has the largest reported mass (about 2000 Earth masses)."
        )));
        assert!(facts.iter().any(|s| s.contains(
            "Among the listed planets, Test c has the smallest reported mass (about 4.5 Earth masses)."
        )));

        let mut hot = h.clone();
        hot["st_teff"] = json!(7400.0);
        let (request, _, _) =
            prepare("Test", &[hot], &[giant.clone(), small.clone()]).unwrap();
        assert!(
            request.publishable_comparisons.iter().any(|s| s
                == "The star's reported temperature is hotter than the Sun's.")
        );

        small["pl_bmassprov"] = json!("Msini");
        let (request, _, _) =
            prepare("Test", &[h], &[giant.clone(), small.clone()]).unwrap();
        let facts = &request.publishable_comparisons;
        assert!(facts.iter().any(|s| s
            == "Test c's reported minimum-mass quantity is larger than Earth's."));
        assert!(
            !facts.iter().any(|s| s.contains("reported mass (about"))
                || !facts
                    .iter()
                    .any(|s| s.contains("minimum-mass quantity (about"))
        );
        let (request, _, _) =
            prepare("Test", &[host()], &[giant.clone(), small.clone()]).unwrap();
        assert!(
            !request
                .publishable_comparisons
                .iter()
                .any(|s| s.contains("has the largest reported")
                    || s.contains("has the smallest reported"))
        );

        giant["pl_bmassprov"] = json!("Msini");
        let (request, _, _) =
            prepare("Test", &[host()], &[giant, small]).unwrap();
        assert!(request.publishable_comparisons.iter().any(|s| s.contains(
            "Test b has the largest reported minimum-mass quantity (about 2000 Earth masses)."
        )));

        let mut overlap = planet("Test d", 30.0);
        overlap["pl_bmasse"] = json!(1.05);
        overlap["pl_bmasselim"] = json!(0);
        overlap["pl_bmasseerr1"] = json!(0.06);
        overlap["pl_bmasseerr2"] = json!(-0.06);
        let (request, _, _) =
            prepare("Test", &[host()], &[planet("Test e", 40.0), overlap])
                .unwrap();
        assert!(
            !request
                .publishable_comparisons
                .iter()
                .any(|s| s.contains("Test d's reported mass"))
        );
    }

    #[test]
    fn second_place_facts_require_three_planets_and_separation() {
        let mut b = planet("Test b", 10.0);
        b["pl_rade"] = json!(1.5);
        b["pl_radelim"] = json!(0);
        b["pl_bmasse"] = json!(2.0);
        b["pl_bmasselim"] = json!(0);
        let mut c = planet("Test c", 20.0);
        c["pl_rade"] = json!(2.5);
        c["pl_radelim"] = json!(0);
        c["pl_bmasse"] = json!(4.5);
        c["pl_bmasselim"] = json!(0);
        let mut d = planet("Test d", 30.0);
        d["pl_rade"] = json!(3.5);
        d["pl_radelim"] = json!(0);
        d["pl_bmasse"] = json!(2000.0);
        d["pl_bmasselim"] = json!(0);
        let (request, _, _) =
            prepare("Test", &[host()], &[b.clone(), c.clone(), d.clone()])
                .unwrap();
        let facts = &request.publishable_comparisons;
        assert!(facts.iter().any(|s| s.contains(
            "Test c has the second-shortest year by reported estimates (about 20 days)."
        )));
        assert!(facts.iter().any(|s| s.contains(
            "Test c has the second-longest year by reported estimates (about 20 days)."
        )));
        assert!(facts.iter().any(|s| s.contains(
            "Test c has the second-smallest radius by reported estimates (about 2.5 Earth radii)."
        )));
        assert!(facts.iter().any(|s| s.contains(
            "Test c has the second-largest reported mass (about 4.5 Earth masses)."
        )));
        let mut tied = planet("Test g", 25.0);
        tied["pl_rade"] = json!(1.5049);
        tied["pl_radelim"] = json!(0);
        let (request, _, _) = prepare("Test", &[host()], &[b, tied, d]).unwrap();
        let facts = &request.publishable_comparisons;
        assert!(facts.iter().any(|s| s.contains(
            "Test d has the largest radius by reported estimates (about 3.5 Earth radii)."
        )));
        assert!(!facts.iter().any(|s| s.contains("smallest radius")
            || s.contains("second-largest radius")));
        let mut two = planet("Test e", 40.0);
        two["pl_rade"] = json!(1.5);
        two["pl_radelim"] = json!(0);
        two["pl_bmasse"] = json!(2.0);
        two["pl_bmasselim"] = json!(0);
        let mut other = planet("Test f", 50.0);
        other["pl_rade"] = json!(3.5);
        other["pl_radelim"] = json!(0);
        other["pl_bmasse"] = json!(4.5);
        other["pl_bmasselim"] = json!(0);
        let (request, _, _) = prepare("Test", &[host()], &[two, other]).unwrap();
        assert!(
            !request
                .publishable_comparisons
                .iter()
                .any(|s| s.contains("second-"))
        );
    }

    #[test]
    fn output_is_deterministic_and_toml_round_trips() {
        let mut rows = vec![planet("Test c", 20.0), planet("Test b", 10.0)];
        let (a, evidence_a, _) = prepare("Test", &[host()], &rows).unwrap();
        rows.reverse();
        let (b, evidence_b, _) = prepare("Test", &[host()], &rows).unwrap();
        let text = toml::to_string_pretty(&a).unwrap();
        assert_eq!(text, toml::to_string_pretty(&b).unwrap());
        assert_eq!(evidence_a, evidence_b);
        let parsed: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(parsed["system"]["hostname"].as_str(), Some("Test"));
        assert_eq!(a.planets[0].name, "Test b");
        assert!(
            a.publishable_comparisons
                .iter()
                .any(|s| s.contains("shortest year"))
        );
        assert!(
            !a.publishable_comparisons
                .iter()
                .any(|s| s.contains("largest radius"))
        );
    }

    #[test]
    fn identifiers_and_significant_digits() {
        assert_eq!(system_id(" HD 41004  A ").unwrap(), "hd-41004-a");
        assert_eq!(system_id("../A\\B\tC!\n").unwrap(), "ab-c");
        assert!(system_id("../☀").is_err());
        assert_eq!(significant(0.00123456), "0.00123");
        assert_eq!(significant(12345.6), "12300");
        assert_eq!(significant(1.23456), "1.23");
        assert_eq!(significant(999.99), "1000");
    }
}
