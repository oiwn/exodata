//! Offline, deterministic evidence preparation for prose experiments.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use exo_core::selection::{
    exoplanet_record_index, stellar_host_record_index, system_identifier,
};
use exo_core::tables::overview::planet_size_class;
use polars::prelude::{ChunkCompareEq, DataFrame, ParquetReader, SerReader};
use serde::Serialize;
use serde_json::{Value, json};

const GUIDE: &str =
    include_str!("../../../../content/prompts/stellarhost_guide.toml");
const LIGHT_YEARS_PER_PARSEC: f64 = 3.26156;
const EARTH_MASSES_PER_JUPITER: f64 = 317.8;
const SUN_EFFECTIVE_TEMPERATURE_K: f64 = 5772.0;
const EARTH_DENSITY_G_CM3: f64 = 5.51;
const VERY_YOUNG_AGE_GYR: f64 = 0.1;

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
    classification: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    circumbinary: Option<bool>,
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
    host_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    age_class: Option<&'static str>,
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
    [
        "hostname",
        "evidence_path",
        "request_path",
        "diagnostics_count",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// Compact single-line outcome for the `lines` output format.
pub fn line(row: &Value) -> String {
    let hostname = row["hostname"].as_str().unwrap_or("?");
    if let Some(error) = row["error"].as_str() {
        let cut: String = error.chars().take(90).collect();
        return format!("{hostname:<26} error     {cut}");
    }
    let count = row["diagnostics_count"].as_u64().unwrap_or_default();
    let mode = if row["dry_run"] == true {
        "dry-run"
    } else {
        "prepared"
    };
    format!("{hostname:<26} {mode:<9} {count} diagnostics")
}

pub fn system_id(hostname: &str) -> Result<String> {
    system_identifier(hostname).ok_or_else(|| {
        anyhow::anyhow!("Hostname produces an empty system identifier")
    })
}

/// Source tables loaded once and shared across many per-hostname
/// preparations, with a precomputed identifier-collision index.
pub struct Catalog {
    data_dir: PathBuf,
    hosts: DataFrame,
    planets: DataFrame,
    collisions: BTreeMap<String, Vec<String>>,
}

impl Catalog {
    pub fn load(data_dir: &Path) -> Result<Self> {
        let host_path = data_dir.join("stellarhosts.parquet");
        let planet_path = data_dir.join("exoplanets.parquet");
        let load = |path: &Path| -> Result<DataFrame> {
            ParquetReader::new(fs::File::open(path)?)
                .finish()
                .with_context(|| format!("Cannot read {}", path.display()))
        };
        let hosts = load(&host_path)?;
        let planets = load(&planet_path)?;
        let mut collisions: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for frame in [&hosts, &planets] {
            let column = frame.column("hostname")?.str()?;
            for name in (0..column.len()).filter_map(|i| column.get(i)) {
                if let Ok(id) = system_id(name) {
                    let entry = collisions.entry(id).or_default();
                    if !entry.iter().any(|stored| stored == name) {
                        entry.push(name.to_owned());
                    }
                }
            }
        }
        Ok(Self {
            data_dir: data_dir.to_owned(),
            hosts,
            planets,
            collisions,
        })
    }

    /// Distinct hostnames with at least one planet row, sorted.
    pub fn all_hostnames(&self) -> Result<Vec<String>> {
        let column = self.planets.column("hostname")?.str()?;
        let names: BTreeSet<String> = (0..column.len())
            .filter_map(|i| column.get(i))
            .filter(|name| !name.trim().is_empty())
            .map(str::to_owned)
            .collect();
        Ok(names.into_iter().collect())
    }

    pub fn prepare(
        &self,
        output_dir: &Path,
        hostname: &str,
        force: bool,
        dry_run: bool,
    ) -> Result<Value> {
        let id = system_id(hostname)?;
        if let Some(other) = self
            .collisions
            .get(&id)
            .filter(|list| list.len() > 1)
            .and_then(|list| list.iter().find(|stored| stored != &hostname))
        {
            bail!(
                "System identifier {id:?} collides for {hostname:?} and {other:?}"
            );
        }
        let host_rows = selected_rows(&self.hosts, hostname)?;
        let planet_rows = selected_rows(&self.planets, hostname)?;
        let (mut request, mut evidence, diagnostics) =
            prepare(hostname, &host_rows, &planet_rows)?;
        let directory = output_dir.join(id);
        if dry_run {
            return Ok(json!({"hostname": hostname,
                "evidence_path": directory.join("evidence.json"),
                "request_path": directory.join("request.toml"),
                "diagnostics": diagnostics,
                "diagnostics_count": diagnostics.len(),
                "dry_run": true}));
        }
        let paths = json!({"stellarhosts": self.data_dir.join("stellarhosts.parquet"),
            "exoplanets": self.data_dir.join("exoplanets.parquet")});
        request.source["files"] = paths.clone();
        evidence["files"] = paths;
        let request_text = toml::to_string_pretty(&request)?;
        let evidence_text = serde_json::to_string_pretty(&evidence)? + "\n";
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
            "request_path": directory.join("request.toml"), "diagnostics": diagnostics,
            "diagnostics_count": diagnostics.len()}),
        )
    }
}

fn selected_rows(frame: &DataFrame, hostname: &str) -> Result<Vec<Value>> {
    let mask = frame.column("hostname")?.str()?.equal(hostname);
    exo_core::json::dataframe_to_json(&frame.filter(&mask)?)
}

pub fn run(
    data_dir: &Path,
    output_dir: &Path,
    hostname: &str,
    force: bool,
) -> Result<Value> {
    Catalog::load(data_dir)?.prepare(output_dir, hostname, force, false)
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
    if !recovery_failed && let Err(error) = fs::remove_dir_all(&staging) {
        return match result {
            Ok(()) => Err(error).context(
                "Preparation pair installed, but staging cleanup failed",
            ),
            Err(previous) => {
                Err(previous
                    .context(format!("Staging cleanup also failed: {error}")))
            }
        };
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
            classification: measurements
                .get("radius")
                .filter(|m| m.interval().is_some())
                .map(|m| planet_size_class(m.value)),
            circumbinary: (row["cb_flag"].as_i64() == Some(1)).then_some(true),
            discovery_method: string(row, "discoverymethod"),
            discovery_year: row["disc_year"].as_i64(),
            measurements,
        });
    }
    let host_kind = (!planets.is_empty()
        && planets
            .iter()
            .any(|p| p.discovery_method.as_deref() == Some("Pulsar Timing")))
    .then_some("pulsar");
    let mut star = Star {
        host_kind,
        age_class: None,
        spectral_type: string(host, "st_spectype"),
        measurements: BTreeMap::new(),
    };
    if star.spectral_type.is_none()
        && let Some(spectype) = host_rows
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
    star.age_class = star
        .measurements
        .get("age")
        .filter(|m| m.qualifier == "estimate")
        .filter(|m| m.value < VERY_YOUNG_AGE_GYR)
        .map(|_| "very young");
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
    let target_words = if comparisons.len() < 10 || planets.len() < 2 {
        "150–300; shorter when needed to avoid repetition or unsupported claims"
    } else {
        "300–600; shorter when needed to avoid repetition or unsupported claims"
    };
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
    if !planets
        .iter()
        .any(|p| p.discovery_method.as_deref() == Some("Radial Velocity"))
    {
        guide.remove("radial_velocity");
    }
    if !planets.iter().any(|p| p.classification.is_some()) {
        guide.remove("planet_classes");
    }
    if !planets.iter().any(|p| p.circumbinary == Some(true)) {
        guide.remove("circumbinary");
    }
    if star.host_kind.is_none() {
        guide.remove("pulsar_timing");
    }
    if star.age_class.is_none() {
        guide.remove("stellar_age");
    }
    if star.spectral_type.is_none() {
        guide.remove("spectral");
    }
    let mut silent_constraints = vec![
        "Only system, star, planets, publishable_comparisons, and guide supply article content; source is audit context.".into(),
        "These constraints and diagnostics are silent instructions, never reader-facing prose.".into(),
        "Missing measurements are unknown. Do not infer composition, density, habitability, orbital spacing, orbital speed, or observing feasibility.".into(),
        "Use only approved comparisons. Do not calculate new ratios or rank masses; preserve minimum-mass provenance and upper/lower limits.".into(),
        "Catalog system counts can include other hosts. Describe only the planets that orbit this hostname; name order is not orbital order.".into(),
    ];
    if system.catalog_system_star_count.is_some_and(|n| n > 1) {
        silent_constraints.push(
            "The stellar measurements describe the selected host star only; never attribute them to the companion star or stars, and never call them system-wide values.".into(),
        );
    }
    let request = Request {
        schema_version: 1,
        request: BTreeMap::from([
            (
                "task",
                "Describe this stellar host and its planets for curious general readers.",
            ),
            ("target_words", target_words),
            (
                "format",
                "A factual Markdown title and connected paragraphs; no tables or bullet lists",
            ),
            (
                "coverage",
                "Introduce the host, name its planets, and select useful supplied measurements. Preserve all qualifiers. Do not recite every number or force a generic ending.",
            ),
        ]),
        guide,
        source,
        system,
        star,
        planets,
        publishable_comparisons: comparisons,
        silent_constraints,
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
        if !planet.measurements.contains_key("orbital_period") {
            result.push(format!(
                "No orbital period is reported for {}.",
                planet.name
            ));
        }
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
        if let (Some(mass), Some(radius)) = (
            planet
                .measurements
                .get("mass")
                .filter(|m| m.interval().is_some())
                .filter(|m| m.provenance.as_deref() == Some("Mass")),
            planet
                .measurements
                .get("radius")
                .filter(|m| m.interval().is_some()),
        ) {
            let relative = mass.value / radius.value.powi(3);
            result.push(format!(
                "{}'s estimated mean density is about {} g/cm³, about {} \
                 times Earth's.",
                planet.name,
                significant(relative * EARTH_DENSITY_G_CM3),
                significant(relative)
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
            // With exactly three ranked planets the second-lowest and
            // second-highest are the same planet; emit it once.
            if values.len() > 3 && distinct_from_all(&values, last - 1) {
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
                // With exactly three ranked planets the second-smallest and
                // second-largest are the same planet; emit it once.
                if masses.len() > 3 && distinct_from_all(&masses, last - 1) {
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
    fn periods_remain_in_days_without_earth_year_comparisons() {
        for period in [5.0, 10.0, 45.0, 365.0, 1095.0, 2000.0] {
            let (request, _, _) = prepare(
                "Test",
                std::slice::from_ref(&host()),
                &[planet("Test b", period)],
            )
            .unwrap();
            assert!(
                !request.publishable_comparisons.iter().any(|s| {
                    crate::descriptions::validate::is_earth_year_reference(s)
                }),
                "period {period}"
            );
            assert_eq!(
                request.planets[0].measurements["orbital_period"].value,
                period
            );
            assert!(
                request.planets[0].measurements["orbital_period"]
                    .display
                    .ends_with(" days")
            );
        }
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
            prepare("Test", std::slice::from_ref(&h), &rows).unwrap();
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
        assert!(prepare("Test", &[], std::slice::from_ref(&p)).is_err());
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
            "Test c has the second-smallest radius by reported estimates (about 2.5 Earth radii)."
        )));
        assert!(facts.iter().any(|s| s.contains(
            "Test c has the second-smallest reported mass (about 4.5 Earth masses)."
        )));
        // With exactly three ranked planets the middle one must not receive
        // paired second-place labels.
        assert!(!facts.iter().any(|s| s.contains("second-longest year")
            || s.contains("second-largest reported mass")));
        let mut fourth = planet("Test h", 60.0);
        fourth["pl_rade"] = json!(4.5);
        fourth["pl_radelim"] = json!(0);
        fourth["pl_bmasse"] = json!(3000.0);
        fourth["pl_bmasselim"] = json!(0);
        let (request, _, _) =
            prepare("Test", &[host()], &[b.clone(), c, d.clone(), fourth])
                .unwrap();
        let facts = &request.publishable_comparisons;
        assert!(facts.iter().any(|s| s.contains("second-longest year")));
        assert!(
            facts
                .iter()
                .any(|s| s.contains("second-smallest reported mass"))
        );
        let mut tied = planet("Test g", 25.0);
        tied["pl_rade"] = json!(1.5049);
        tied["pl_radelim"] = json!(0);
        let (request, _, _) =
            prepare("Test", &[host()], &[b.clone(), tied, d.clone()]).unwrap();
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
    fn density_and_classification_follow_provenance() {
        let mut b = planet("Test b", 10.0);
        b["pl_rade"] = json!(2.0);
        b["pl_radelim"] = json!(0);
        b["pl_bmasse"] = json!(16.0);
        b["pl_bmasselim"] = json!(0);
        let mut c = planet("Test c", 20.0);
        c["pl_rade"] = json!(3.0);
        c["pl_radelim"] = json!(0);
        c["pl_bmasse"] = json!(50.0);
        c["pl_bmasselim"] = json!(0);
        c["pl_bmassprov"] = json!("Msini");
        let (request, _, _) = prepare("Test", &[host()], &[b, c]).unwrap();
        assert_eq!(request.planets[0].classification, Some("Super-Earth"));
        assert_eq!(request.planets[1].classification, Some("Neptune-like"));
        assert!(request.guide.contains_key("planet_classes"));
        assert!(request.publishable_comparisons.iter().any(|s| s.contains(
            "Test b's estimated mean density is about 11 g/cm³, about 2 times Earth's."
        )));
        assert!(
            !request
                .publishable_comparisons
                .iter()
                .any(|s| s.contains("Test c's estimated mean density"))
        );
        let (request, _, _) =
            prepare("Test", &[host()], &[planet("Test b", 10.0)]).unwrap();
        assert_eq!(request.planets[0].classification, None);
        assert!(!request.guide.contains_key("planet_classes"));
    }

    #[test]
    fn cb_flag_host_kind_age_class_and_missing_period_license_facts() {
        let mut young = host();
        young["st_age"] = json!(0.03);
        young["st_agelim"] = json!(0);
        let mut b = planet("Test b", 10.0);
        b["cb_flag"] = json!(1);
        b["discoverymethod"] = json!("Pulsar Timing");
        let mut c = planet("Test c", 20.0);
        c["cb_flag"] = json!(0);
        c["discoverymethod"] = json!("Pulsar Timing");
        let (request, _, _) = prepare("Test", &[young], &[b, c.clone()]).unwrap();
        assert_eq!(request.planets[0].circumbinary, Some(true));
        assert_eq!(request.planets[1].circumbinary, None);
        assert_eq!(request.star.host_kind, Some("pulsar"));
        assert_eq!(request.star.age_class, Some("very young"));
        assert!(request.guide.contains_key("circumbinary"));
        assert!(request.guide.contains_key("pulsar_timing"));
        assert!(request.guide.contains_key("stellar_age"));

        let mut binary = host();
        binary["sy_snum"] = json!(2);
        let mut no_flag = planet("Test b", 10.0);
        no_flag["discoverymethod"] = json!("Transit");
        let (request, _, _) =
            prepare("Test", &[binary], &[no_flag.clone()]).unwrap();
        assert_eq!(request.planets[0].circumbinary, None);
        assert_eq!(request.star.host_kind, None);
        assert_eq!(request.star.age_class, None);
        assert!(!request.guide.contains_key("circumbinary"));
        assert!(!request.guide.contains_key("pulsar_timing"));
        assert!(!request.guide.contains_key("stellar_age"));

        let mut no_period = no_flag;
        no_period["pl_orbper"] = Value::Null;
        let (request, _, _) = prepare("Test", &[host()], &[no_period]).unwrap();
        assert!(
            request
                .publishable_comparisons
                .iter()
                .any(|s| s == "No orbital period is reported for Test b.")
        );
        let (request, _, _) = prepare("Test", &[host()], &[c]).unwrap();
        assert!(
            !request
                .publishable_comparisons
                .iter()
                .any(|s| s.contains("No orbital period"))
        );
    }

    #[test]
    fn multi_star_systems_constrain_stellar_attribution() {
        let mut binary = host();
        binary["sy_snum"] = json!(2);
        let (request, _, _) =
            prepare("Test", &[binary], &[planet("Test b", 10.0)]).unwrap();
        assert!(
            request
                .silent_constraints
                .iter()
                .any(|s| s.contains("describe the selected host star only"))
        );
        let (request, _, _) =
            prepare("Test", &[host()], &[planet("Test b", 10.0)]).unwrap();
        assert!(
            !request
                .silent_constraints
                .iter()
                .any(|s| s.contains("selected host star"))
        );
    }

    #[test]
    fn target_words_follows_fact_richness() {
        let mut b = planet("Test b", 10.0);
        b["pl_rade"] = json!(2.5);
        b["pl_radelim"] = json!(0);
        b["pl_bmasse"] = json!(4.5);
        b["pl_bmasselim"] = json!(0);
        let mut c = planet("Test c", 20.0);
        c["pl_rade"] = json!(3.5);
        c["pl_radelim"] = json!(0);
        c["pl_bmasse"] = json!(2000.0);
        c["pl_bmasselim"] = json!(0);
        let (request, _, _) = prepare("Test", &[host()], &[b, c]).unwrap();
        assert!(
            request.publishable_comparisons.len() >= 10,
            "fixture should be fact-rich, got {}",
            request.publishable_comparisons.len()
        );
        assert_eq!(
            request.request["target_words"],
            "300–600; shorter when needed to avoid repetition or unsupported claims"
        );
        let (request, _, _) =
            prepare("Test", &[host()], &[planet("Test b", 10.0)]).unwrap();
        assert_eq!(
            request.request["target_words"],
            "150–300; shorter when needed to avoid repetition or unsupported claims"
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
