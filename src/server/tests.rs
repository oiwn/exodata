#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use polars::prelude::*;
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::server::cache::build_table_cache;
    use crate::server::functions::DataStats;
    use crate::server::handlers::{
        ApiState, get_exoplanets, get_exoplanets_schema, get_stellarhosts,
        get_stellarhosts_schema,
    };

    fn create_test_state() -> ApiState {
        // Create test dataframes with all default columns that common.rs expects
        let stellarhosts_df = df! {
            "hostname" => &["HD 189733", "Kepler-22", "HD 209458"],
            "sy_dist" => &[19.3, 600.0, 47.9],
            "st_teff" => &[5040.0, 5518.0, 6092.0],
            "st_mass" => &[0.82, 0.97, 1.01],
            "sy_pnum" => &[1i64, 1i64, 1i64],
        }
        .unwrap();

        let exoplanets_df = df! {
            "pl_name" => &["HD 189733 b", "Kepler-22 b", "HD 209458 b"],
            "hostname" => &["HD 189733", "Kepler-22", "HD 209458"],
            "discoverymethod" => &["Radial Velocity", "Transit", "Transit"],
            "disc_year" => &[2005i64, 2011i64, 1999i64],
            "pl_orbper" => &[2.218, 289.9, 3.524],
            "pl_rade" => &[1.138, 2.38, 1.32],
            "pl_bmasse" => &[1.15, 2.25, 0.69],
        }
        .unwrap();

        ApiState {
            stellarhosts_df: Arc::new(stellarhosts_df),
            exoplanets_df: Arc::new(exoplanets_df),
            stellarhosts_metadata: Arc::new(HashMap::new()),
            exoplanets_metadata: Arc::new(HashMap::new()),
            overview_stats: Arc::new(DataStats {
                stellarhosts_total: 0,
                exoplanets_total: 0,
                avg_stellar_temp: 0.0,
                avg_stellar_distance: 0.0,
                discovery_methods: Vec::new(),
                planet_size_categories: Vec::new(),
            }),
            table_cache: build_table_cache(64),
        }
    }

    fn create_test_app() -> Router {
        let state = create_test_state();
        Router::new()
            .route("/rest/stellarhosts", axum::routing::get(get_stellarhosts))
            .route("/rest/exoplanets", axum::routing::get(get_exoplanets))
            .route(
                "/rest/stellarhosts/schema",
                axum::routing::get(get_stellarhosts_schema),
            )
            .route(
                "/rest/exoplanets/schema",
                axum::routing::get(get_exoplanets_schema),
            )
            .with_state(state)
    }

    #[tokio::test]
    async fn test_get_stellarhosts_default() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/stellarhosts")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        assert_eq!(json["total"], 3);
        assert_eq!(json["total_all"], 3);
        assert_eq!(json["page"], 1);
        assert_eq!(json["limit"], 50);

        // Check that default columns are returned
        let columns = json["columns"].as_array().unwrap();
        let column_names: Vec<&str> =
            columns.iter().filter_map(|c| c.as_str()).collect();
        assert!(column_names.contains(&"hostname"));
        assert!(column_names.contains(&"sy_dist"));
    }

    #[tokio::test]
    async fn test_get_stellarhosts_with_columns() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/stellarhosts?columns=hostname,st_teff")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Check only requested columns are in response
        let columns = json["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 2);

        let column_names: Vec<&str> =
            columns.iter().filter_map(|c| c.as_str()).collect();
        assert!(column_names.contains(&"hostname"));
        assert!(column_names.contains(&"st_teff"));
        assert!(!column_names.contains(&"sy_dist"));
    }

    #[tokio::test]
    async fn test_get_stellarhosts_with_pagination() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/stellarhosts?page=1&limit=2")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 2);
        assert_eq!(json["total"], 3);
        assert_eq!(json["total_all"], 3);
        assert_eq!(json["page"], 1);
        assert_eq!(json["limit"], 2);
    }

    #[tokio::test]
    async fn test_get_stellarhosts_with_sorting() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/stellarhosts?sort_by=sy_dist&order=asc")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let data = json["data"].as_array().unwrap();
        assert_eq!(data[0]["hostname"], "HD 189733"); // dist = 19.3
        assert_eq!(data[1]["hostname"], "HD 209458"); // dist = 47.9
        assert_eq!(data[2]["hostname"], "Kepler-22"); // dist = 600.0
    }

    #[tokio::test]
    async fn test_get_exoplanets_default() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/exoplanets")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        assert_eq!(json["total"], 3);
        assert_eq!(json["total_all"], 3);

        // Check that default columns are returned
        let columns = json["columns"].as_array().unwrap();
        let column_names: Vec<&str> =
            columns.iter().filter_map(|c| c.as_str()).collect();
        assert!(column_names.contains(&"pl_name"));
        assert!(column_names.contains(&"hostname"));
    }

    #[tokio::test]
    async fn test_get_exoplanets_with_columns() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/exoplanets?columns=pl_name,disc_year")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Check only requested columns
        let columns = json["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 2);

        let column_names: Vec<&str> =
            columns.iter().filter_map(|c| c.as_str()).collect();
        assert!(column_names.contains(&"pl_name"));
        assert!(column_names.contains(&"disc_year"));
    }

    #[tokio::test]
    async fn test_get_exoplanets_with_sorting() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/exoplanets?sort_by=disc_year&order=asc")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let data = json["data"].as_array().unwrap();
        assert_eq!(data[0]["pl_name"], "HD 209458 b"); // 1999
        assert_eq!(data[1]["pl_name"], "HD 189733 b"); // 2005
        assert_eq!(data[2]["pl_name"], "Kepler-22 b"); // 2011
    }

    #[tokio::test]
    async fn test_get_stellarhosts_schema() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/stellarhosts/schema")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json["columns"].is_array());
        assert_eq!(json["total_rows"], 3);

        let columns = json["columns"].as_array().unwrap();
        let column_names: Vec<String> = columns
            .iter()
            .map(|col| col["name"].as_str().unwrap().to_string())
            .collect();

        assert!(column_names.contains(&"hostname".to_string()));
        assert!(column_names.contains(&"sy_dist".to_string()));
        assert!(column_names.contains(&"st_teff".to_string()));
    }

    #[tokio::test]
    async fn test_get_exoplanets_schema() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/exoplanets/schema")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json["columns"].is_array());
        assert_eq!(json["total_rows"], 3);

        let columns = json["columns"].as_array().unwrap();
        let column_names: Vec<String> = columns
            .iter()
            .map(|col| col["name"].as_str().unwrap().to_string())
            .collect();

        assert!(column_names.contains(&"pl_name".to_string()));
        assert!(column_names.contains(&"hostname".to_string()));
        assert!(column_names.contains(&"pl_orbper".to_string()));
    }

    #[tokio::test]
    async fn test_limit_capped_at_1000() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/stellarhosts?limit=5000")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Limit should be capped at 1000
        assert_eq!(json["limit"], 1000);
    }
}
