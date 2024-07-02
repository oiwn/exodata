#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use clap::Parser;
    use exoplanets_catalog::common;
    use exoplanets_catalog::stellarhosts;

    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about = None)]
    struct Cli {
        #[clap(subcommand)]
        command: Option<Commands>,
    }

    #[derive(Parser, Debug)]
    enum Commands {
        Serve,
        ImportData,
        ViewFields { path: String }, // Print VOTable fields
        CodegenVotable { name: String, path: String }, // Genrate rust structure from VOTable fields
        Check,                                         // Check something
    }

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Serve) | None => {
            start_server().await;
        }
        Some(Commands::ImportData) => {
            let _ = stellarhosts::load_data();
        }
        Some(Commands::ViewFields { path }) => {
            common::print_votable_headers(path);
        }
        Some(Commands::CodegenVotable { name, path }) => {
            let _ = common::structure_from_votables_codegen(path, name);
        }
        Some(Commands::Check) => {
            common::extract_coumns_types("data/stellarhosts.vot");

            // let nullable_columns = common::detect_nullable_columns("data/stellarhosts.vot");
            // println!("{:?}", nullable_columns);
            // println!("LEN. {}", nullable_columns.len());
        }
    }
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
    use exoplanets_catalog::app::*;
    use exoplanets_catalog::fileserv::file_and_error_handler;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our application with a route
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
