use exo_types::insights::InsightMeta;
use polars::prelude::*;
use polars::sql::SQLContext;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsightError {
    #[error("unknown insight slug '{0}'")]
    UnknownSlug(String),
    #[error("failed to execute insight SQL for {slug}")]
    Execute {
        slug: String,
        #[source]
        source: PolarsError,
    },
    #[error("failed to collect insight {slug}")]
    Collect {
        slug: String,
        #[source]
        source: PolarsError,
    },
}

pub mod binary_systems;
pub mod crowded_systems;
pub mod distant_exoplanets;
pub mod equal_star_planet_pairs;
pub mod hottest_stellar_hosts;
pub mod largest_exoplanets;
pub mod nearest_stellar_hosts;
pub mod planet_host_ratios;
pub mod smallest_exoplanets;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsightTable {
    Exoplanets,
    StellarHosts,
    Both,
}

#[derive(Clone, Copy, Debug)]
pub struct InsightDef {
    pub meta: &'static InsightMeta,
    pub table: InsightTable,
    pub sql: &'static str,
}

pub struct InsightInput<'a> {
    pub stellarhosts: &'a DataFrame,
    pub exoplanets: &'a DataFrame,
}

pub struct InsightData {
    pub slug: &'static str,
    pub columns: Vec<String>,
    pub frame: DataFrame,
}

pub static INSIGHTS: &[&InsightDef] = &[
    &smallest_exoplanets::DEF,
    &largest_exoplanets::DEF,
    &distant_exoplanets::DEF,
    &nearest_stellar_hosts::DEF,
    &planet_host_ratios::DEF,
    &equal_star_planet_pairs::DEF,
    &hottest_stellar_hosts::DEF,
    &crowded_systems::DEF,
    &binary_systems::DEF,
];

pub fn find_insight(slug: &str) -> Option<&'static InsightDef> {
    INSIGHTS.iter().copied().find(|def| def.meta.slug == slug)
}

pub fn run_insight(
    input: InsightInput<'_>,
    slug: &str,
) -> Result<InsightData, InsightError> {
    let def = find_insight(slug)
        .ok_or_else(|| InsightError::UnknownSlug(slug.to_string()))?;
    run_insight_def(input, def)
}

pub fn run_insight_def(
    input: InsightInput<'_>,
    def: &'static InsightDef,
) -> Result<InsightData, InsightError> {
    let mut ctx = SQLContext::new();

    match def.table {
        InsightTable::Exoplanets => {
            ctx.register("exoplanets", input.exoplanets.clone().lazy());
        }
        InsightTable::StellarHosts => {
            ctx.register("stellarhosts", input.stellarhosts.clone().lazy());
        }
        InsightTable::Both => {
            ctx.register("stellarhosts", input.stellarhosts.clone().lazy());
            ctx.register("exoplanets", input.exoplanets.clone().lazy());
        }
    }

    let frame = ctx
        .execute(def.sql)
        .map_err(|source| InsightError::Execute {
            slug: def.meta.slug.to_string(),
            source,
        })?
        .collect()
        .map_err(|source| InsightError::Collect {
            slug: def.meta.slug.to_string(),
            source,
        })?;
    let columns = frame
        .get_column_names()
        .iter()
        .map(|name| name.to_string())
        .collect();

    Ok(InsightData {
        slug: def.meta.slug,
        columns,
        frame,
    })
}

pub fn run_all_insights(
    input: InsightInput<'_>,
) -> Result<Vec<InsightData>, InsightError> {
    INSIGHTS
        .iter()
        .map(|&def| {
            run_insight_def(
                InsightInput {
                    stellarhosts: input.stellarhosts,
                    exoplanets: input.exoplanets,
                },
                def,
            )
        })
        .collect()
}
