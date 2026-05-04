#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    start_server().await;
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

#[cfg(feature = "ssr")]
const BUILD_TIMESTAMP: &str = match option_env!("BUILD_TIMESTAMP") {
    Some(value) => value,
    None => "unknown",
};

#[cfg(feature = "ssr")]
const BUILD_DATE: &str = match option_env!("BUILD_DATE") {
    Some(value) => value,
    None => "",
};

#[cfg(feature = "ssr")]
fn compute_build_date(timestamp: &str, explicit_date: &str) -> String {
    if !explicit_date.is_empty() {
        return explicit_date.to_string();
    }
    if timestamp != "unknown" && !timestamp.is_empty() {
        return timestamp.split(' ').next().unwrap_or("").to_string();
    }
    // Fallback: use current UTC date for local dev
    use time::OffsetDateTime;
    OffsetDateTime::now_utc()
        .format(&time::macros::format_description!("[year]-[month]-[day]"))
        .unwrap_or_default()
}

#[cfg(feature = "ssr")]
async fn start_server() {
    use axum::Router;
    use exo_core::metadata;
    use exo_core::tables::common as data_common;
    use exo_core::tables::overview as aggregation;
    use exoplanets_catalog::app::{App, shell};
    use exoplanets_catalog::metadata::AppMetadata;
    use exoplanets_catalog::server::data::{
        insights as server_insights, tables as server_tables,
    };
    use exoplanets_catalog::server::functions::{
        ColumnMetadata as UiColumnMetadata, DataStats,
    };
    use exoplanets_catalog::server::{self, ApiState};
    use leptos::prelude::{get_configuration, provide_context};
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Instant;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load dataframes at startup
    let stellarhosts_df =
        match data_common::load_parquet("data/stellarhosts.parquet", None) {
            Ok(df) => Arc::new(df),
            Err(e) => panic!("Failed to load stellarhosts data: {}", e),
        };

    let exoplanets_df =
        match data_common::load_parquet("data/exoplanets.parquet", None) {
            Ok(df) => Arc::new(df),
            Err(e) => panic!("Failed to load exoplanets data: {}", e),
        };

    // Load metadata from TOML files
    let stellarhosts_metadata = match metadata::load_metadata_toml(Path::new(
        "data/stellarhosts-metadata.toml",
    )) {
        Ok(meta) => Arc::new(meta),
        Err(e) => panic!("Failed to load stellarhosts metadata: {}", e),
    };

    let exoplanets_metadata = match metadata::load_metadata_toml(Path::new(
        "data/exoplanets-metadata.toml",
    )) {
        Ok(meta) => Arc::new(meta),
        Err(e) => panic!("Failed to load exoplanets metadata: {}", e),
    };
    let app_metadata = AppMetadata {
        stellarhosts: stellarhosts_metadata
            .as_ref()
            .iter()
            .map(|(k, v)| (k.clone(), UiColumnMetadata::from(v.clone())))
            .collect(),
        exoplanets: exoplanets_metadata
            .as_ref()
            .iter()
            .map(|(k, v)| (k.clone(), UiColumnMetadata::from(v.clone())))
            .collect(),
    };
    let metadata_json = leptos::serde_json::to_string(&app_metadata)
        .expect("Failed to serialize metadata for hydration");

    // Precompute overview stats for cache-backed access.
    let (stellarhosts_total, exoplanets_total) =
        aggregation::get_total_counts(&stellarhosts_df, &exoplanets_df);
    let avg_stellar_temp =
        aggregation::get_avg_temperature(&stellarhosts_df).unwrap_or(0.0);
    let avg_stellar_distance =
        aggregation::get_avg_distance(&stellarhosts_df).unwrap_or(0.0);
    let discovery_methods =
        aggregation::get_discovery_methods(&exoplanets_df, 10);
    let planet_size_categories =
        aggregation::get_planet_size_categories(&exoplanets_df);
    let discovery_years =
        aggregation::get_discovery_year_counts(&exoplanets_df, 10);
    let orbital_period_buckets =
        aggregation::get_orbital_period_buckets(&exoplanets_df);
    let overview_stats = Arc::new(DataStats {
        stellarhosts_total,
        exoplanets_total,
        avg_stellar_temp,
        avg_stellar_distance,
        discovery_methods,
        planet_size_categories,
        discovery_years,
        orbital_period_buckets,
    });

    let table_cache = server::cache::build_table_cache(400);
    let host_detail_cache = server::cache::build_host_detail_cache(512);
    let insight_cache = server::cache::build_insight_cache(32);
    let site_url = Arc::new(
        std::env::var("SITE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "https://exodata.space".to_string()),
    );
    let build_date = compute_build_date(BUILD_TIMESTAMP, BUILD_DATE);
    let sitemaps = Arc::new(
        server::handlers::build_sitemaps(
            site_url.as_str(),
            &build_date,
            &stellarhosts_df,
            &exoplanets_df,
        )
        .unwrap_or_else(|e| panic!("Failed to build sitemaps: {}", e)),
    );

    let api_state = ApiState {
        site_url,
        sitemap_index_xml: Arc::new(sitemaps.index.clone()),
        sitemap_static_xml: Arc::new(sitemaps.static_pages.clone()),
        sitemap_stellarhosts_xml: Arc::new(sitemaps.stellarhosts.clone()),
        sitemap_exoplanets_xml: Arc::new(sitemaps.exoplanets.clone()),
        stellarhosts_df,
        exoplanets_df,
        stellarhosts_metadata,
        exoplanets_metadata,
        overview_stats,
        table_cache,
        host_detail_cache,
        insight_cache,
    };

    // Prewarm default table cache entries before serving any requests.
    let prewarm_started = Instant::now();
    server_tables::get_stellarhosts_data_cached(
        &api_state.stellarhosts_df,
        &api_state.table_cache,
        server_tables::TableQuery {
            page: 1,
            limit: 50,
            sort_by: None,
            order: None,
            selected_columns: None,
            filter: None,
        },
    )
    .await
    .unwrap_or_else(|e| {
        panic!(
            "Startup prewarm failed for stellarhosts default page: {}",
            e
        )
    });
    server_tables::get_exoplanets_data_cached(
        &api_state.exoplanets_df,
        &api_state.table_cache,
        server_tables::TableQuery {
            page: 1,
            limit: 50,
            sort_by: None,
            order: None,
            selected_columns: None,
            filter: None,
        },
    )
    .await
    .unwrap_or_else(|e| {
        panic!("Startup prewarm failed for exoplanets default page: {}", e)
    });
    tracing::info!(
        "table cache prewarm complete in {:?}",
        prewarm_started.elapsed()
    );

    let insight_prewarm_started = Instant::now();
    for def in exo_core::insights::INSIGHTS {
        server_insights::get_insight_cached(
            &api_state.stellarhosts_df,
            &api_state.exoplanets_df,
            &api_state.insight_cache,
            def.meta.slug,
        )
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Startup prewarm failed for insight {}: {}",
                def.meta.slug, e
            )
        });
    }
    tracing::info!(
        "insight cache prewarm complete in {:?}",
        insight_prewarm_started.elapsed()
    );

    let ga_measurement_id = std::env::var("LEPTOS_GA_ID").ok();

    // Local dev: use Cargo.toml (via cargo-leptos)
    // Production: use LEPTOS_* environment variables
    // See: https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain
    let conf = if std::path::Path::new("Cargo.toml").exists() {
        get_configuration(Some("Cargo.toml"))
    } else {
        get_configuration(None)
    }
    .expect("Failed to load Leptos configuration");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Create a shared context provider closure for both SSR and server functions
    let provide_api_state = {
        let api_state = api_state.clone();
        let app_metadata = app_metadata.clone();
        move || {
            provide_context(api_state.clone());
            provide_context(app_metadata.clone());
        }
    };

    // Build Leptos app first
    let app: Router = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            provide_api_state.clone(),
            {
                let leptos_options = leptos_options.clone();
                let ga_measurement_id = ga_measurement_id.clone();
                let metadata_json = metadata_json.clone();
                move || {
                    shell(
                        leptos_options.clone(),
                        ga_measurement_id.clone(),
                        metadata_json.clone(),
                    )
                }
            },
        )
        .fallback(leptos_axum::file_and_error_handler({
            let ga_measurement_id = ga_measurement_id.clone();
            let metadata_json = metadata_json.clone();
            move |options| {
                shell(options, ga_measurement_id.clone(), metadata_json.clone())
            }
        }))
        .with_state(leptos_options);

    // Merge REST API and Swagger UI on top (these take priority over Leptos fallback)
    let app = app
        .merge(server::site_routes(api_state.clone()))
        .merge(server::swagger_ui()) // Swagger UI at /swagger-ui
        .nest_service("/mcp", server::mcp::mcp_routes(api_state.clone())) // MCP Streamable HTTP
        .nest_service("/rest", server::api_routes(api_state)); // REST API at /rest/*

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
