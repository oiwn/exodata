use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct TableResponse {
    pub data: Vec<Value>,
    pub columns: Vec<String>,
    pub total: usize,
    pub total_all: usize,
    pub page: usize,
    pub limit: usize,
}

#[derive(Debug, Deserialize)]
pub struct SqlResponse {
    pub data: Vec<Value>,
    pub rows: usize,
    pub columns: Vec<String>,
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct SchemaResponse {
    pub columns: Vec<ColumnInfo>,
    pub total_rows: usize,
}

#[derive(Debug, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InsightResponse {
    pub meta: InsightMeta,
    pub data: Vec<Value>,
    pub rows: usize,
    pub columns: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InsightMeta {
    pub slug: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub kind: String,
    pub limit: usize,
}

pub struct ApiClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new(base_url: String, timeout_seconds: u64) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()?;
        Ok(Self { base_url, client })
    }

    pub fn query(&self, sql: &str, limit: Option<usize>) -> Result<SqlResponse> {
        let mut request = self
            .client
            .get(self.url("/rest/query"))
            .query(&[("sql", sql)]);
        if let Some(limit) = limit {
            request = request.query(&[("limit", limit)]);
        }
        self.send(request)
    }

    pub fn rows(&self, params: RowsRequest<'_>) -> Result<TableResponse> {
        let mut request = self
            .client
            .get(self.url(&format!("/rest/{}", params.table)));
        request =
            request.query(&[("page", params.page), ("limit", params.limit)]);
        if let Some(sort_by) = params.sort_by {
            request = request.query(&[("sort_by", sort_by)]);
        }
        if let Some(order) = params.order {
            request = request.query(&[("order", order)]);
        }
        if let Some(columns) = params.columns {
            request = request.query(&[("columns", columns)]);
        }
        if let Some(filter) = params.filter {
            request = request.query(&[("filter", filter)]);
        }
        self.send(request)
    }

    pub fn schema(&self, table: &str) -> Result<SchemaResponse> {
        self.send(self.client.get(self.url(&format!("/rest/{table}/schema"))))
    }

    pub fn insights(&self) -> Result<Vec<InsightMeta>> {
        self.send(self.client.get(self.url("/rest/insights"))).with_context(
            || {
                "failed to list API insights; if the deployed server is older, use the default auto backend or --backend local".to_string()
            },
        )
    }

    pub fn insight(&self, slug: &str) -> Result<InsightResponse> {
        self.send(self.client.get(self.url(&format!("/rest/insights/{slug}"))))
            .with_context(|| {
                "failed to run API insight; if the deployed server is older, use --backend local with a complete local dataset".to_string()
            })
    }

    pub fn download_file(&self, source_path: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(self.url(source_path))
            .send()
            .with_context(|| format!("failed to GET {}", source_path))?
            .error_for_status()
            .with_context(|| format!("download failed for {}", source_path))?;
        Ok(response.bytes()?.to_vec())
    }

    fn send<T: for<'de> Deserialize<'de>>(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> Result<T> {
        Ok(request.send()?.error_for_status()?.json()?)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

pub struct RowsRequest<'a> {
    pub table: &'a str,
    pub page: usize,
    pub limit: usize,
    pub sort_by: Option<&'a str>,
    pub order: Option<&'a str>,
    pub columns: Option<&'a str>,
    pub filter: Option<&'a str>,
}
