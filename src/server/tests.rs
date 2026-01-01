#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use polars::prelude::*;
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::server::handlers::{ApiState, get_stellarhosts, get_exoplanets, get_stellarhosts_schema, get_exoplanets_schema};

    fn create_test_state() -> ApiState {
        // Create test dataframes
        let stellarhosts_df = df! {
            "hostname" => &["HD 189733", "Kepler-22", "HD 209458"],
            "sy_dist" => &[19.3, 600.0, 47.9],
            "st_teff" => &[5040.0, 5518.0, 6092.0],
            "sy_pnum" => &[1, 1, 1],
        }
        .unwrap();

        let exoplanets_df = df! {
            "pl_name" => &["HD 189733 b", "Kepler-22 b", "HD 209458 b"],
            "hostname" => &["HD 189733", "Kepler-22", "HD 209458"],
            "pl_orbper" => &[2.218, 289.9, 3.524],
            "pl_rade" => &[1.138, 2.38, 1.32],
            "pl_masse" => &[1.15, 2.25, 0.69],
        }
        .unwrap();

        ApiState {
            stellarhosts_df: Arc::new(stellarhosts_df),
            exoplanets_df: Arc::new(exoplanets_df),
            stellarhosts_metadata: Arc::new(HashMap::new()),
            exoplanets_metadata: Arc::new(HashMap::new()),
        }
    }

    fn create_test_app() -> Router {
        // Using a custom router since we need to override with test data
        let state = create_test_state();
        Router::new()
            .route("/api/stellarhosts", axum::routing::get(get_stellarhosts))
            .route("/api/exoplanets", axum::routing::get(get_exoplanets))
            .route("/api/stellarhosts/schema", axum::routing::get(get_stellarhosts_schema))
            .route("/api/exoplanets/schema", axum::routing::get(get_exoplanets_schema))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_get_stellarhosts_all() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/stellarhosts")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        assert_eq!(json["total"], 3);
        assert_eq!(json["page"], 1);
        assert_eq!(json["limit"], 50);
    }

    #[tokio::test]
    async fn test_get_stellarhosts_with_filter() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/stellarhosts?hostname=HD%20189733")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 1);
        assert_eq!(json["total"], 1);
        assert_eq!(json["data"][0]["hostname"], "HD 189733");
    }

    #[tokio::test]
    async fn test_get_stellarhosts_with_pagination() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/stellarhosts?page=1&limit=2")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 2);
        assert_eq!(json["total"], 3);
        assert_eq!(json["page"], 1);
        assert_eq!(json["limit"], 2);
    }

    #[tokio::test]
    async fn test_get_stellarhosts_with_sorting() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/stellarhosts?sort_by=sy_dist&order=asc")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let data = json["data"].as_array().unwrap();
        assert_eq!(data[0]["hostname"], "HD 189733"); // dist = 19.3
        assert_eq!(data[1]["hostname"], "HD 209458"); // dist = 47.9
        assert_eq!(data[2]["hostname"], "Kepler-22");  // dist = 600.0
    }

    #[tokio::test]
    async fn test_get_exoplanets_all() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/exoplanets")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        assert_eq!(json["total"], 3);
    }

    #[tokio::test]
    async fn test_get_exoplanets_with_filter() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/exoplanets?pl_name=Kepler-22%20b")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 1);
        assert_eq!(json["total"], 1);
        assert_eq!(json["data"][0]["pl_name"], "Kepler-22 b");
    }

    #[tokio::test]
    async fn test_get_exoplanets_with_numeric_filter() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/exoplanets?pl_orbper_min=2.5")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Should include Kepler-22 b (289.9) and HD 209458 b (3.524), but not HD 189733 b (2.218)
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
        assert_eq!(json["total"], 2);
    }

    #[tokio::test]
    async fn test_get_stellarhosts_schema() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/api/stellarhosts/schema")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json["columns"].is_array());
        assert_eq!(json["total_rows"], 3);
        
        let columns = json["columns"].as_array().unwrap();
        let column_names: Vec<String> = columns.iter()
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
            .uri("/api/exoplanets/schema")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(json["columns"].is_array());
        assert_eq!(json["total_rows"], 3);
        
        let columns = json["columns"].as_array().unwrap();
        let column_names: Vec<String> = columns.iter()
            .map(|col| col["name"].as_str().unwrap().to_string())
            .collect();
        
        assert!(column_names.contains(&"pl_name".to_string()));
        assert!(column_names.contains(&"hostname".to_string()));
        assert!(column_names.contains(&"pl_orbper".to_string()));
    }
}