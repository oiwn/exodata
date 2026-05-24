use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use exo_core::metadata::ColumnMetadata;
use polars::prelude::*;
use polars::sql::SQLContext;
use serde::Serialize;
use serde_json::Value;
use sqlparser::ast::{SetExpr, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

use super::rows;

#[derive(Debug, Clone, Serialize)]
pub struct CatalogQueryResult {
    pub data: Vec<Value>,
    pub rows: usize,
    pub columns: Vec<String>,
    pub query: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogDescription {
    pub table: String,
    pub total_rows: usize,
    pub columns: Vec<CatalogColumn>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogColumn {
    pub name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub source_datatype: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogSqlError {
    Empty,
    Invalid(String),
    Execution(String),
    Timeout,
    Join(String),
}

impl fmt::Display for CatalogSqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogSqlError::Empty => f.write_str("SQL query must not be empty"),
            CatalogSqlError::Invalid(message) => f.write_str(message),
            CatalogSqlError::Execution(message) => f.write_str(message),
            CatalogSqlError::Timeout => f.write_str("SQL query timed out"),
            CatalogSqlError::Join(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for CatalogSqlError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogSchemaError {
    UnknownTable(String),
    UnknownColumns { table: String, columns: Vec<String> },
}

impl fmt::Display for CatalogSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogSchemaError::UnknownTable(table) => {
                write!(f, "unknown catalog table: {table}")
            }
            CatalogSchemaError::UnknownColumns { table, columns } => {
                write!(
                    f,
                    "unknown columns for table {table}: {}",
                    columns.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for CatalogSchemaError {}

pub async fn execute_catalog_sql(
    stellarhosts_df: Arc<DataFrame>,
    exoplanets_df: Arc<DataFrame>,
    query: String,
    limit: usize,
) -> Result<CatalogQueryResult, CatalogSqlError> {
    let query = query.trim().to_string();
    validate_sql_select_only(&query)?;

    let query_for_exec = query.clone();
    let handle = tokio::task::spawn_blocking(move || {
        let mut ctx = SQLContext::new();
        ctx.register("stellarhosts", stellarhosts_df.as_ref().clone().lazy());
        ctx.register("exoplanets", exoplanets_df.as_ref().clone().lazy());

        let lazy = ctx.execute(&query_for_exec).map_err(|e| {
            CatalogSqlError::Execution(format!("SQL execution error: {e}"))
        })?;
        let df = lazy.limit(limit as IdxSize).collect().map_err(|e| {
            CatalogSqlError::Execution(format!(
                "Failed to collect SQL result: {e}"
            ))
        })?;

        let rows = df.height();
        let columns = df
            .get_column_names()
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>();
        let data =
            rows::dataframe_to_json(&df).map_err(CatalogSqlError::Execution)?;

        Ok::<_, CatalogSqlError>((data, columns, rows))
    });

    let result = tokio::time::timeout(Duration::from_secs(30), handle).await;
    match result {
        Err(_) => Err(CatalogSqlError::Timeout),
        Ok(join_result) => match join_result {
            Err(err) => Err(CatalogSqlError::Join(format!(
                "SQL query join error: {err}"
            ))),
            Ok(Err(err)) => Err(err),
            Ok(Ok((data, columns, rows))) => Ok(CatalogQueryResult {
                data,
                rows,
                columns,
                query,
            }),
        },
    }
}

pub fn validate_sql_select_only(query: &str) -> Result<(), CatalogSqlError> {
    if query.trim().is_empty() {
        return Err(CatalogSqlError::Empty);
    }

    let dialect = GenericDialect {};
    let statements = Parser::parse_sql(&dialect, query).map_err(|e| {
        CatalogSqlError::Invalid(format!("invalid SQL syntax: {e}"))
    })?;

    if statements.len() != 1 {
        return Err(CatalogSqlError::Invalid(
            "SQL must contain exactly one statement".to_string(),
        ));
    }

    match statements.first() {
        Some(Statement::Query(query)) if is_select_query(&query.body) => Ok(()),
        _ => Err(CatalogSqlError::Invalid(
            "only SELECT statements are allowed".to_string(),
        )),
    }
}

pub fn describe_catalog(
    table: &str,
    df: &DataFrame,
    metadata: &HashMap<String, ColumnMetadata>,
    columns: Option<&[String]>,
) -> Result<CatalogDescription, CatalogSchemaError> {
    let requested = columns.map(|cols| {
        cols.iter()
            .map(|column| column.trim())
            .filter(|column| !column.is_empty())
            .collect::<Vec<_>>()
    });

    let fields = df.fields();
    let field_names = fields
        .iter()
        .map(|field| field.name().as_str())
        .collect::<Vec<_>>();

    let selected = match requested {
        Some(columns) if !columns.is_empty() => {
            let unknown = columns
                .iter()
                .filter(|column| !field_names.contains(column))
                .map(|column| (*column).to_string())
                .collect::<Vec<_>>();

            if !unknown.is_empty() {
                return Err(CatalogSchemaError::UnknownColumns {
                    table: table.to_string(),
                    columns: unknown,
                });
            }

            columns
        }
        _ => field_names,
    };

    let columns = selected
        .into_iter()
        .filter_map(|name| {
            let field =
                fields.iter().find(|field| field.name().as_str() == name)?;
            let meta = metadata.get(name);
            Some(CatalogColumn {
                name: name.to_string(),
                data_type: format!("{:?}", field.dtype()),
                description: meta.and_then(|m| m.description.clone()),
                unit: meta.and_then(|m| m.unit.clone()),
                source_datatype: meta.map(|m| m.datatype.clone()),
            })
        })
        .collect();

    Ok(CatalogDescription {
        table: table.to_string(),
        total_rows: df.height(),
        columns,
    })
}

fn is_select_query(expr: &SetExpr) -> bool {
    match expr {
        SetExpr::Select(_) => true,
        SetExpr::Query(query) => is_select_query(&query.body),
        SetExpr::SetOperation { left, right, .. } => {
            is_select_query(left) && is_select_query(right)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_sql_select_only_accepts_single_select() {
        assert!(
            validate_sql_select_only(
                "SELECT pl_name, hostname FROM exoplanets LIMIT 10"
            )
            .is_ok()
        );
    }

    #[test]
    fn validate_sql_select_only_rejects_empty_sql() {
        assert_eq!(validate_sql_select_only(""), Err(CatalogSqlError::Empty));
    }

    #[test]
    fn validate_sql_select_only_rejects_multiple_statements() {
        assert!(matches!(
            validate_sql_select_only(
                "SELECT * FROM exoplanets; SELECT * FROM stellarhosts"
            ),
            Err(CatalogSqlError::Invalid(_))
        ));
    }

    #[test]
    fn validate_sql_select_only_rejects_non_select_statement() {
        assert!(matches!(
            validate_sql_select_only("DELETE FROM exoplanets"),
            Err(CatalogSqlError::Invalid(_))
        ));
    }

    #[test]
    fn validate_sql_select_only_rejects_values_query() {
        assert!(matches!(
            validate_sql_select_only("VALUES (1)"),
            Err(CatalogSqlError::Invalid(_))
        ));
    }

    #[test]
    fn describe_catalog_filters_requested_columns() {
        let df = df! {
            "pl_name" => &["Kepler-22 b"],
            "pl_rade" => &[2.38],
        }
        .unwrap();
        let mut metadata = HashMap::new();
        metadata.insert(
            "pl_rade".to_string(),
            ColumnMetadata {
                name: "pl_rade".to_string(),
                description: Some("Planet Radius".to_string()),
                unit: Some("Rearth".to_string()),
                datatype: "double".to_string(),
            },
        );

        let columns = vec!["pl_rade".to_string()];
        let description =
            describe_catalog("exoplanets", &df, &metadata, Some(&columns))
                .unwrap();

        assert_eq!(description.table, "exoplanets");
        assert_eq!(description.total_rows, 1);
        assert_eq!(description.columns.len(), 1);
        assert_eq!(description.columns[0].name, "pl_rade");
        assert_eq!(description.columns[0].unit.as_deref(), Some("Rearth"));
        assert_eq!(
            description.columns[0].source_datatype.as_deref(),
            Some("double")
        );
    }

    #[test]
    fn describe_catalog_rejects_unknown_columns() {
        let df = df! {
            "pl_name" => &["Kepler-22 b"],
        }
        .unwrap();
        let columns = vec!["missing".to_string()];

        assert!(matches!(
            describe_catalog("exoplanets", &df, &HashMap::new(), Some(&columns)),
            Err(CatalogSchemaError::UnknownColumns { .. })
        ));
    }
}
