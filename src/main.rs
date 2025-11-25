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
        ImportData { path: String },
        ViewFields { path: String }, // Print VOTable fields
        CodegenVotable { name: String, path: String }, // Genrate rust structure from VOTable fields
        Check,                                         // Check something
    }

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Serve) | None => {
            start_server().await;
        }
        Some(Commands::ImportData { path }) => {
            println!("Importing data from {}...", path);
            match stellarhosts::load_data(path) {
                Ok(df) => {
                    println!("Successfully loaded data.");
                    println!("DataFrame shape: {:?}", df.shape());
                }
                Err(e) => {
                    eprintln!("Error loading data: {}", e);
                }
            }
        }
        Some(Commands::ViewFields { path }) => {
            common::print_votable_headers(path);
        }
        Some(Commands::CodegenVotable { name, path }) => {
            let _ = common::structure_from_votables_codegen(path, name);
        }
        Some(Commands::Check) => {
            match stellarhosts::load_data("data/stellarhosts.vot") {
                Ok(df) => {
                    println!("Successfully loaded data.");
                    println!("{}", df);
                }
                Err(e) => {
                    eprintln!("Error loading data: {}", e);
                }
            }
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
    use exoplanets_catalog::app::{shell, App};
    use leptos::prelude::get_configuration; // Added this import
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

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
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
