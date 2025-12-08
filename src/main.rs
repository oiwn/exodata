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
    use exoplanets_catalog::tables::common;
    use leptos::prelude::{get_configuration, provide_context};
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::sync::Arc;

    // Load dataframes at startup
    let stellarhosts_df = match common::load_parquet("data/stellarhosts.parquet", None) {
        Ok(df) => Arc::new(df),
        Err(e) => panic!("Failed to load stellarhosts data: {}", e),
    };

    let exoplanets_df = match common::load_parquet("data/exoplanets.parquet", None) {
        Ok(df) => Arc::new(df),
        Err(e) => panic!("Failed to load exoplanets data: {}", e),
    };

    let api_state = ApiState {
        stellarhosts_df,
        exoplanets_df,
    };

    // Setting get_configuration(Some("Cargo.toml")) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    let conf = get_configuration(Some("Cargo.toml")).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our application with a route
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            let api_state = api_state.clone();
            move || {
                provide_context(api_state.clone());
                shell(leptos_options.clone())
            }
        })
        .nest_service("/api", server::api_routes(api_state))
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
