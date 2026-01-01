#[cfg(feature = "ssr")]
#[tokio::main]
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
    use exoplanets_catalog::app::{shell, App};
    use exoplanets_catalog::server::{self, ApiState};
    use exo_core::metadata;
    use exo_core::tables::common;
    use leptos::prelude::{get_configuration, provide_context};
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::path::Path;
    use std::sync::Arc;

    // Load dataframes at startup
    let stellarhosts_df =
        match common::load_parquet("data/stellarhosts.parquet", None) {
            Ok(df) => Arc::new(df),
            Err(e) => panic!("Failed to load stellarhosts data: {}", e),
        };

    let exoplanets_df =
        match common::load_parquet("data/exoplanets.parquet", None) {
            Ok(df) => Arc::new(df),
            Err(e) => panic!("Failed to load exoplanets data: {}", e),
        };

    // Load metadata from TOML files
    let stellarhosts_metadata = match metadata::load_metadata_toml(
        Path::new("data/stellarhosts-metadata.toml")
    ) {
        Ok(meta) => Arc::new(meta),
        Err(e) => panic!("Failed to load stellarhosts metadata: {}", e),
    };

    let exoplanets_metadata = match metadata::load_metadata_toml(
        Path::new("data/exoplanets-metadata.toml")
    ) {
        Ok(meta) => Arc::new(meta),
        Err(e) => panic!("Failed to load exoplanets metadata: {}", e),
    };

    let api_state = ApiState {
        stellarhosts_df,
        exoplanets_df,
        stellarhosts_metadata,
        exoplanets_metadata,
    };

    // Setting get_configuration(Some("Cargo.toml")) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    let conf = get_configuration(Some("Cargo.toml")).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Create a shared context provider closure for both SSR and server functions
    let provide_api_state = {
        let api_state = api_state.clone();
        move || {
            provide_context(api_state.clone());
        }
    };

    // build our application with a route
    let app = Router::new()
        // leptos_routes_with_context handles both SSR and server functions automatically
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            provide_api_state.clone(),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .nest_service("/rest", server::api_routes(api_state)) // Axum REST API at /rest/* (Leptos uses /api/*)
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
