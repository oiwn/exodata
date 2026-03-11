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
async fn start_server() {
    use axum::Router;
    use exo_core::metadata;
    use exo_core::tables::common as data_common;
    use exo_core::tables::overview as aggregation;
    use exoplanets_catalog::app::{App, shell};
    use exoplanets_catalog::metadata::AppMetadata;
    use exoplanets_catalog::server::common as server_common;
    use exoplanets_catalog::server::functions::{
        ColumnMetadata as UiColumnMetadata, DataStats,
    };
    use exoplanets_catalog::server::{self, ApiState};
    use leptos::prelude::{get_configuration, provide_context};
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Instant;

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
    let overview_stats = Arc::new(DataStats {
        stellarhosts_total,
        exoplanets_total,
        avg_stellar_temp,
        avg_stellar_distance,
        discovery_methods,
        planet_size_categories,
    });

    let table_cache = server::cache::build_table_cache(400);

    let api_state = ApiState {
        stellarhosts_df,
        exoplanets_df,
        stellarhosts_metadata,
        exoplanets_metadata,
        overview_stats,
        table_cache,
    };

    // Prewarm default table cache entries before serving any requests.
    let prewarm_started = Instant::now();
    server_common::get_stellarhosts_data_cached(
        &api_state.stellarhosts_df,
        &api_state.stellarhosts_metadata,
        &api_state.table_cache,
        1,
        50,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap_or_else(|e| {
        panic!(
            "Startup prewarm failed for stellarhosts default page: {}",
            e
        )
    });
    server_common::get_exoplanets_data_cached(
        &api_state.exoplanets_df,
        &api_state.exoplanets_metadata,
        &api_state.table_cache,
        1,
        50,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap_or_else(|e| {
        panic!("Startup prewarm failed for exoplanets default page: {}", e)
    });
    println!(
        "table cache prewarm complete in {:?}",
        prewarm_started.elapsed()
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
        .merge(server::swagger_ui()) // Swagger UI at /swagger-ui
        .nest_service("/rest", server::api_routes(api_state)); // REST API at /rest/*

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
