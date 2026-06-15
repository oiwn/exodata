#[cfg(test)]
#[allow(clippy::module_inception)]
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

    use crate::server::cache::{
        build_host_detail_cache, build_insight_cache, build_table_cache,
    };
    use crate::server::functions::DataStats;
    use crate::server::handlers::{
        ApiState, build_sitemaps, get_exoplanets, get_exoplanets_schema,
        get_stellarhosts, get_stellarhosts_schema, site_routes,
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

        let sitemaps = build_sitemaps(
            "https://example.com",
            "2026-01-15",
            &stellarhosts_df,
            &exoplanets_df,
        )
        .unwrap();

        ApiState {
            site_url: Arc::new("https://example.com".to_string()),
            sitemap_index_xml: Arc::new(sitemaps.index),
            sitemap_static_xml: Arc::new(sitemaps.static_pages),
            sitemap_entity_xml: Arc::new(
                sitemaps
                    .entity_sitemaps
                    .into_iter()
                    .map(|(filename, xml)| (filename, Arc::new(xml)))
                    .collect(),
            ),
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
                planet_temperature_bands: Vec::new(),
                detection_sources: Vec::new(),
                discovery_years: Vec::new(),
                orbital_period_buckets: Vec::new(),
            }),
            table_cache: build_table_cache(64),
            host_detail_cache: build_host_detail_cache(64),
            insight_cache: build_insight_cache(16),
        }
    }

    fn create_test_app() -> Router {
        let state = create_test_state();
        site_routes(state.clone()).merge(
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
                .with_state(state),
        )
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
    async fn test_get_stellarhosts_page_zero_reports_page_one() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/rest/stellarhosts?page=0&limit=2")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"].as_array().unwrap().len(), 2);
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

    #[tokio::test]
    async fn test_sitemap_index() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/sitemap-index.xml")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/xml; charset=utf-8"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let xml = String::from_utf8(body.to_vec()).unwrap();

        assert!(xml.contains("<sitemapindex"));
        assert!(xml.contains("https://example.com/sitemap-static.xml"));
        assert!(xml.contains("https://example.com/sitemap-stellarhosts-1.xml"));
        assert!(xml.contains("https://example.com/sitemap-exoplanets-1.xml"));
        assert!(!xml.contains("https://example.com/sitemap-stellarhosts.xml"));
        assert!(!xml.contains("https://example.com/sitemap-exoplanets.xml"));
        assert!(xml.contains("<lastmod>2026-01-15</lastmod>"));
    }

    #[tokio::test]
    async fn test_sitemap_static() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/sitemap-static.xml")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let xml = String::from_utf8(body.to_vec()).unwrap();

        assert!(xml.contains("<urlset"));
        assert!(xml.contains("<loc>https://example.com/</loc>"));
        assert!(xml.contains("<loc>https://example.com/docs</loc>"));
        assert!(xml.contains("<loc>https://example.com/docs/cli</loc>"));
        assert!(xml.contains("<loc>https://example.com/docs/api</loc>"));
        assert!(xml.contains("<loc>https://example.com/stellarhosts</loc>"));
        assert!(xml.contains("<loc>https://example.com/exoplanets</loc>"));
        assert!(xml.contains("<loc>https://example.com/insights</loc>"));
        assert!(xml.contains(
            "<loc>https://example.com/insights/smallest-exoplanets-radius</loc>"
        ));
        assert!(xml.contains(
            "<loc>https://example.com/insights/largest-exoplanets-radius</loc>"
        ));
        assert!(xml.contains(
            "<loc>https://example.com/insights/most-distant-exoplanets</loc>"
        ));
        assert!(xml.contains(
            "<loc>https://example.com/insights/nearest-stellar-hosts</loc>"
        ));
        assert!(xml.contains(
            "<loc>https://example.com/insights/largest-planet-to-host-ratios</loc>"
        ));
        assert!(xml.contains(
            "<loc>https://example.com/insights/most-equal-star-planet-pairs</loc>"
        ));
        assert!(xml.contains(
            "<loc>https://example.com/insights/binary-star-systems</loc>"
        ));
        assert!(xml.contains("<lastmod>2026-01-15</lastmod>"));
        assert!(!xml.contains("/stellarhosts/"));
        assert!(!xml.contains("/exoplanets/"));
    }

    #[tokio::test]
    async fn test_sitemap_stellarhosts() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/sitemap-stellarhosts-1.xml")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let xml = String::from_utf8(body.to_vec()).unwrap();

        assert!(xml.contains("<urlset"));
        assert!(
            xml.contains(
                "<loc>https://example.com/stellarhosts/HD%20189733</loc>"
            )
        );
        assert!(xml.contains("<lastmod>2026-01-15</lastmod>"));
    }

    #[tokio::test]
    async fn test_sitemap_exoplanets() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/sitemap-exoplanets-1.xml")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let xml = String::from_utf8(body.to_vec()).unwrap();

        assert!(xml.contains("<urlset"));
        assert!(xml.contains(
            "<loc>https://example.com/exoplanets/HD%20189733%20b</loc>"
        ));
        assert!(xml.contains("<lastmod>2026-01-15</lastmod>"));
    }

    #[tokio::test]
    async fn test_old_entity_sitemap_routes_return_not_found() {
        let app = create_test_app();

        for uri in ["/sitemap-stellarhosts.xml", "/sitemap-exoplanets.xml"] {
            let request =
                Request::builder().uri(uri).body(Body::empty()).unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }
    }

    #[tokio::test]
    async fn test_unknown_entity_sitemap_chunk_returns_not_found() {
        let app = create_test_app();

        let request = Request::builder()
            .uri("/sitemap-exoplanets-999.xml")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_sitemap_entity_chunks_are_limited_and_indexed() {
        let hostnames = (0..1001)
            .map(|idx| format!("Host {idx}"))
            .collect::<Vec<_>>();
        let planet_names = (0..1001)
            .map(|idx| format!("Planet {idx} b"))
            .collect::<Vec<_>>();
        let stellarhosts_df = df! {
            "hostname" => hostnames,
        }
        .unwrap();
        let exoplanets_df = df! {
            "pl_name" => planet_names,
        }
        .unwrap();

        let sitemaps = build_sitemaps(
            "https://example.com",
            "2026-01-15",
            &stellarhosts_df,
            &exoplanets_df,
        )
        .unwrap();

        assert!(
            sitemaps
                .index
                .contains("https://example.com/sitemap-static.xml")
        );
        assert!(
            sitemaps
                .index
                .contains("https://example.com/sitemap-stellarhosts-1.xml")
        );
        assert!(
            sitemaps
                .index
                .contains("https://example.com/sitemap-stellarhosts-2.xml")
        );
        assert!(
            sitemaps
                .index
                .contains("https://example.com/sitemap-exoplanets-1.xml")
        );
        assert!(
            sitemaps
                .index
                .contains("https://example.com/sitemap-exoplanets-2.xml")
        );
        assert!(!sitemaps.index.contains("sitemap-stellarhosts.xml"));
        assert!(!sitemaps.index.contains("sitemap-exoplanets.xml"));

        for xml in sitemaps.entity_sitemaps.values() {
            assert!(xml.matches("<url>").count() <= 1_000);
        }
    }

    #[test]
    fn test_sitemap_detail_urls_dedupe_and_use_path_segment_encoding() {
        let stellarhosts_df = df! {
            "hostname" => &["Kepler-55", "Kepler-55", "A/B?c#d&e%f"],
        }
        .unwrap();
        let exoplanets_df = df! {
            "pl_name" => &["51 Eri b", "51 Eri b", "Kepler-55 b"],
        }
        .unwrap();

        let sitemaps = build_sitemaps(
            "https://example.com",
            "2026-01-15",
            &stellarhosts_df,
            &exoplanets_df,
        )
        .unwrap();
        let stellarhosts_xml = sitemaps
            .entity_sitemaps
            .get("sitemap-stellarhosts-1.xml")
            .unwrap();
        let exoplanets_xml = sitemaps
            .entity_sitemaps
            .get("sitemap-exoplanets-1.xml")
            .unwrap();

        assert_eq!(
            stellarhosts_xml.matches("/stellarhosts/Kepler-55").count(),
            1
        );
        assert_eq!(
            exoplanets_xml.matches("/exoplanets/51%20Eri%20b").count(),
            1
        );
        assert!(exoplanets_xml.contains("/exoplanets/Kepler-55%20b"));
        assert!(stellarhosts_xml.contains("/stellarhosts/A%2FB%3Fc%23d%26e%25f"));
    }
}
