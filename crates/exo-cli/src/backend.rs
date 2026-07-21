use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use exo_core::metadata::load_metadata_toml;
use exo_core::tables::common::load_parquet;
use exo_types::metadata::ColumnMetadata;
use polars::prelude::*;
use polars::sql::SQLContext;
use serde_json::{Value, json};

use crate::api::{ApiClient, RowsRequest};
use crate::config::{Backend, Config, base_dir};
use crate::output;

#[derive(Clone, Debug, ValueEnum)]
pub enum DatasetKind {
    Stellarhosts,
    Exoplanets,
}

impl DatasetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatasetKind::Stellarhosts => "stellarhosts",
            DatasetKind::Exoplanets => "exoplanets",
        }
    }
}

pub struct RowsQuery {
    pub dataset: DatasetKind,
    pub page: usize,
    pub limit: usize,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub columns: Option<String>,
    pub filter: Option<String>,
}

pub struct RowsResponse {
    pub data: Vec<Value>,
    pub columns: Vec<String>,
}

pub struct SchemaResponse {
    pub columns: Vec<ColumnInfo>,
}

pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub unit: Option<String>,
}

pub struct SqlResponse {
    pub data: Vec<Value>,
    pub columns: Vec<String>,
}

pub struct InsightMeta {
    pub slug: String,
    pub title: String,
    pub category: String,
    pub kind: String,
    pub limit: usize,
}

pub struct InsightResponse {
    pub data: Vec<Value>,
    pub columns: Vec<String>,
}

/// Interface available for CLI tool
pub trait CatalogBackend {
    fn rows(&self, query: RowsQuery) -> Result<RowsResponse>;
    fn schema(&self, dataset: DatasetKind) -> Result<SchemaResponse>;
    fn sql(&self, query: &str, limit: Option<usize>) -> Result<SqlResponse>;
    fn insights_list(&self) -> Result<Vec<InsightMeta>>;
    fn insight_run(&self, slug: &str) -> Result<InsightResponse>;
}

/// Kinda proxy for different types of backends, currently there are 2:
/// `api` and `local`
pub enum ResolvedBackend {
    Api(ApiBackend),
    Local(LocalBackend),
}

impl CatalogBackend for ResolvedBackend {
    fn rows(&self, query: RowsQuery) -> Result<RowsResponse> {
        match self {
            ResolvedBackend::Api(backend) => backend.rows(query),
            ResolvedBackend::Local(backend) => backend.rows(query),
        }
    }

    fn schema(&self, dataset: DatasetKind) -> Result<SchemaResponse> {
        match self {
            ResolvedBackend::Api(backend) => backend.schema(dataset),
            ResolvedBackend::Local(backend) => backend.schema(dataset),
        }
    }

    fn sql(&self, query: &str, limit: Option<usize>) -> Result<SqlResponse> {
        match self {
            ResolvedBackend::Api(backend) => backend.sql(query, limit),
            ResolvedBackend::Local(backend) => backend.sql(query, limit),
        }
    }

    fn insights_list(&self) -> Result<Vec<InsightMeta>> {
        match self {
            ResolvedBackend::Api(backend) => backend.insights_list(),
            ResolvedBackend::Local(backend) => backend.insights_list(),
        }
    }

    fn insight_run(&self, slug: &str) -> Result<InsightResponse> {
        match self {
            ResolvedBackend::Api(backend) => backend.insight_run(slug),
            ResolvedBackend::Local(backend) => backend.insight_run(slug),
        }
    }
}

/// Resolves the configured backend into an API or local implementation.
pub fn resolve_backend(
    config: &Config,
    backend_flag: Option<Backend>,
    data_dir_flag: Option<String>,
    api_base_url: Option<String>,
) -> Result<ResolvedBackend> {
    let requested = config.backend(backend_flag);
    match requested {
        Backend::Api => {
            Ok(ResolvedBackend::Api(ApiBackend::new(config, api_base_url)?))
        }
        Backend::Local => Ok(ResolvedBackend::Local(LocalBackend::new(
            resolve_local_manifest(config, data_dir_flag, true)?
                .ok_or_else(|| anyhow!("local dataset is unavailable"))?,
        )?)),
        Backend::Auto => {
            if let Some(manifest) =
                resolve_local_manifest(config, data_dir_flag, false)?
            {
                Ok(ResolvedBackend::Local(LocalBackend::new(manifest)?))
            } else {
                Ok(ResolvedBackend::Api(ApiBackend::new(config, api_base_url)?))
            }
        }
    }
}

pub struct DatasetManifest {
    pub root: PathBuf,
    pub stellarhosts_parquet: PathBuf,
    pub exoplanets_parquet: PathBuf,
    pub stellarhosts_metadata: PathBuf,
    pub exoplanets_metadata: PathBuf,
}

impl DatasetManifest {
    fn from_root(root: PathBuf) -> Self {
        Self {
            stellarhosts_parquet: root.join("stellarhosts.parquet"),
            exoplanets_parquet: root.join("exoplanets.parquet"),
            stellarhosts_metadata: root.join("stellarhosts-metadata.toml"),
            exoplanets_metadata: root.join("exoplanets-metadata.toml"),
            root,
        }
    }

    fn missing_files(&self) -> Vec<PathBuf> {
        [
            &self.stellarhosts_parquet,
            &self.exoplanets_parquet,
            &self.stellarhosts_metadata,
            &self.exoplanets_metadata,
        ]
        .into_iter()
        .filter(|path| !path.exists())
        .map(|path| path.to_path_buf())
        .collect()
    }

    fn is_complete(&self) -> bool {
        self.missing_files().is_empty()
    }

    fn parquet_path(&self, dataset: &DatasetKind) -> &Path {
        match dataset {
            DatasetKind::Stellarhosts => &self.stellarhosts_parquet,
            DatasetKind::Exoplanets => &self.exoplanets_parquet,
        }
    }

    fn metadata_path(&self, dataset: &DatasetKind) -> &Path {
        match dataset {
            DatasetKind::Stellarhosts => &self.stellarhosts_metadata,
            DatasetKind::Exoplanets => &self.exoplanets_metadata,
        }
    }
}

/// Finds a complete local dataset manifest from CLI, environment, or config paths.
fn resolve_local_manifest(
    config: &Config,
    data_dir_flag: Option<String>,
    strict: bool,
) -> Result<Option<DatasetManifest>> {
    let mut candidates = Vec::new();
    if let Some(path) =
        data_dir_flag.or_else(|| std::env::var("EXO_DATA_DIR").ok())
    {
        candidates.push(PathBuf::from(path));
    } else {
        candidates.push(PathBuf::from(&config.local.data_dir));
        candidates.push(base_dir()?);
    }

    candidates.dedup();
    let manifests = candidates
        .into_iter()
        .map(DatasetManifest::from_root)
        .collect::<Vec<_>>();

    if let Some(manifest) =
        manifests.iter().find(|manifest| manifest.is_complete())
    {
        return Ok(Some(DatasetManifest::from_root(manifest.root.clone())));
    }

    if strict {
        let details = manifests
            .iter()
            .flat_map(|manifest| {
                manifest
                    .missing_files()
                    .into_iter()
                    .map(|path| format!("  - {}", path.display()))
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Err(anyhow!(
            "local dataset is incomplete; missing required files:\n{}",
            details
        ));
    }

    Ok(None)
}

pub struct ApiBackend {
    client: ApiClient,
}

impl ApiBackend {
    fn new(config: &Config, base_url: Option<String>) -> Result<Self> {
        Ok(Self {
            client: ApiClient::new(
                config.api_base_url(base_url),
                config.api.timeout_seconds,
            )?,
        })
    }
}

impl CatalogBackend for ApiBackend {
    fn rows(&self, query: RowsQuery) -> Result<RowsResponse> {
        let response = self.client.rows(RowsRequest {
            table: query.dataset.as_str(),
            page: query.page,
            limit: query.limit,
            sort_by: query.sort_by.as_deref(),
            order: query.order.as_deref(),
            columns: query.columns.as_deref(),
            filter: query.filter.as_deref(),
        })?;
        Ok(RowsResponse {
            data: response.data,
            columns: response.columns,
        })
    }

    fn schema(&self, dataset: DatasetKind) -> Result<SchemaResponse> {
        let response = self.client.schema(dataset.as_str())?;
        Ok(SchemaResponse {
            columns: response
                .columns
                .into_iter()
                .map(|column| ColumnInfo {
                    name: column.name,
                    data_type: column.data_type,
                    description: column.description,
                    unit: column.unit,
                })
                .collect(),
        })
    }

    fn sql(&self, query: &str, limit: Option<usize>) -> Result<SqlResponse> {
        let response = self.client.query(query, limit)?;
        Ok(SqlResponse {
            data: response.data,
            columns: response.columns,
        })
    }

    fn insights_list(&self) -> Result<Vec<InsightMeta>> {
        Ok(self
            .client
            .insights()?
            .into_iter()
            .map(|meta| InsightMeta {
                slug: meta.slug,
                title: meta.title,
                category: meta.category,
                kind: meta.kind,
                limit: meta.limit,
            })
            .collect())
    }

    fn insight_run(&self, slug: &str) -> Result<InsightResponse> {
        let response = self.client.insight(slug)?;
        Ok(InsightResponse {
            data: response.data,
            columns: response.columns,
        })
    }
}

pub struct LocalBackend {
    manifest: DatasetManifest,
}

impl LocalBackend {
    fn new(manifest: DatasetManifest) -> Result<Self> {
        let missing = manifest.missing_files();
        if !missing.is_empty() {
            return Err(anyhow!(
                "local dataset is incomplete; missing required files:\n{}",
                missing
                    .iter()
                    .map(|path| format!("  - {}", path.display()))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        Ok(Self { manifest })
    }

    fn load_dataset(&self, dataset: &DatasetKind) -> Result<DataFrame> {
        load_parquet(
            path_string(self.manifest.parquet_path(dataset)).as_str(),
            None,
        )
        .with_context(|| {
            format!(
                "failed to load {}",
                self.manifest.parquet_path(dataset).display()
            )
        })
    }

    fn load_metadata(
        &self,
        dataset: &DatasetKind,
    ) -> Result<std::collections::HashMap<String, ColumnMetadata>> {
        load_metadata_toml(self.manifest.metadata_path(dataset)).map_err(|err| {
            anyhow!(
                "failed to load {}: {}",
                self.manifest.metadata_path(dataset).display(),
                err
            )
        })
    }
}

impl CatalogBackend for LocalBackend {
    fn rows(&self, query: RowsQuery) -> Result<RowsResponse> {
        let mut df = self.load_dataset(&query.dataset)?;
        let selected_columns =
            selected_columns(&df, &query.dataset, query.columns.as_deref())?;
        df = df.select(selected_columns.clone())?;

        if let Some(filter) = query.filter {
            let needle = filter.trim().to_lowercase();
            if !needle.is_empty()
                && let Some(first_column) = selected_columns.first()
            {
                let series = df.column(first_column)?;
                let series = if matches!(series.dtype(), DataType::String) {
                    series.clone()
                } else {
                    series.cast(&DataType::String)?
                };
                let utf8 = series.str()?;
                let mask: BooleanChunked = utf8
                    .into_iter()
                    .map(|value| {
                        value.map(|s| s.to_lowercase().contains(&needle))
                    })
                    .collect();
                df = df.filter(&mask)?;
            }
        }

        if let Some(sort_by) = &query.sort_by
            && selected_columns.iter().any(|column| column == sort_by)
        {
            let descending = matches!(query.order.as_deref(), Some(order) if order.eq_ignore_ascii_case("desc"));
            df = df.lazy().filter(col(sort_by).is_not_null()).collect()?;
            df = df.sort(
                [sort_by.as_str()],
                SortMultipleOptions::new().with_order_descending(descending),
            )?;
        }

        let page = query.page.max(1);
        let offset = (page - 1) * query.limit;
        df = if offset < df.height() {
            df.slice(offset as i64, query.limit.min(df.height() - offset))
        } else {
            df.slice(0, 0)
        };

        Ok(RowsResponse {
            data: output::dataframe_to_json(&df)?,
            columns: selected_columns,
        })
    }

    fn schema(&self, dataset: DatasetKind) -> Result<SchemaResponse> {
        let df = self.load_dataset(&dataset)?;
        let metadata = self.load_metadata(&dataset)?;
        let columns = df
            .fields()
            .iter()
            .map(|field| {
                let name = field.name().to_string();
                let meta = metadata.get(&name);
                ColumnInfo {
                    name,
                    data_type: format!("{:?}", field.dtype()),
                    description: meta.and_then(|m| m.description.clone()),
                    unit: meta.and_then(|m| m.unit.clone()),
                }
            })
            .collect();
        Ok(SchemaResponse { columns })
    }

    fn sql(&self, query: &str, limit: Option<usize>) -> Result<SqlResponse> {
        let mut ctx = SQLContext::new();
        ctx.register(
            "stellarhosts",
            self.load_dataset(&DatasetKind::Stellarhosts)?.lazy(),
        );
        ctx.register(
            "exoplanets",
            self.load_dataset(&DatasetKind::Exoplanets)?.lazy(),
        );
        let mut df = ctx
            .execute(query)
            .map_err(|err| anyhow!("SQL error: {}", err))?
            .collect()
            .map_err(|err| anyhow!("failed to collect result: {}", err))?;
        if let Some(limit) = limit {
            df = df.slice(0, limit);
        }
        let columns = df
            .get_column_names()
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        Ok(SqlResponse {
            data: output::dataframe_to_json(&df)?,
            columns,
        })
    }

    fn insights_list(&self) -> Result<Vec<InsightMeta>> {
        Ok(compiled_insight_meta())
    }

    fn insight_run(&self, slug: &str) -> Result<InsightResponse> {
        let stellarhosts = self.load_dataset(&DatasetKind::Stellarhosts)?;
        let exoplanets = self.load_dataset(&DatasetKind::Exoplanets)?;
        let data = exo_core::insights::run_insight(
            exo_core::insights::InsightInput {
                stellarhosts: &stellarhosts,
                exoplanets: &exoplanets,
            },
            slug,
        )?;
        let columns = data
            .columns
            .into_iter()
            .filter(|column| column != "host_link_hostname")
            .collect::<Vec<_>>();
        let frame = data.frame.select(&columns)?;
        Ok(InsightResponse {
            data: output::dataframe_to_json(&frame)?,
            columns,
        })
    }
}

fn selected_columns(
    df: &DataFrame,
    dataset: &DatasetKind,
    columns: Option<&str>,
) -> Result<Vec<String>> {
    if let Some(columns) = columns {
        let selected = columns
            .split(',')
            .map(str::trim)
            .filter(|column| !column.is_empty())
            .filter(|column| df.column(column).is_ok())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return Err(anyhow!("no valid columns selected"));
        }
        return Ok(selected);
    }

    let defaults = match dataset {
        DatasetKind::Stellarhosts => {
            ["hostname", "sy_dist", "st_teff", "st_mass", "sy_pnum"].as_slice()
        }
        DatasetKind::Exoplanets => [
            "pl_name",
            "hostname",
            "discoverymethod",
            "disc_year",
            "pl_orbper",
            "pl_rade",
            "pl_bmasse",
        ]
        .as_slice(),
    };

    let selected = defaults
        .iter()
        .filter(|column| df.column(column).is_ok())
        .map(|column| (*column).to_string())
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(anyhow!("no default columns are available"));
    }
    Ok(selected)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn insight_meta_rows(meta: Vec<InsightMeta>) -> (Vec<Value>, Vec<String>) {
    let rows = meta
        .into_iter()
        .map(|meta| {
            json!({
                "slug": meta.slug,
                "title": meta.title,
                "category": meta.category,
                "kind": meta.kind,
                "limit": meta.limit,
            })
        })
        .collect::<Vec<_>>();
    let columns = vec![
        "slug".to_string(),
        "title".to_string(),
        "category".to_string(),
        "kind".to_string(),
        "limit".to_string(),
    ];
    (rows, columns)
}

pub fn compiled_insight_meta() -> Vec<InsightMeta> {
    exo_core::insights::INSIGHTS
        .iter()
        .map(|def| InsightMeta {
            slug: def.meta.slug.to_string(),
            title: def.meta.title.to_string(),
            category: def.meta.category.to_string(),
            kind: def.meta.kind.to_string(),
            limit: def.meta.limit,
        })
        .collect()
}

pub fn schema_rows(schema: SchemaResponse) -> (Vec<Value>, Vec<String>) {
    let rows = schema
        .columns
        .into_iter()
        .map(|column| {
            json!({
                "name": column.name,
                "data_type": column.data_type,
                "description": column.description,
                "unit": column.unit,
            })
        })
        .collect::<Vec<_>>();
    let columns = vec![
        "name".to_string(),
        "data_type".to_string(),
        "description".to_string(),
        "unit".to_string(),
    ];
    (rows, columns)
}
