#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use clap::Parser;
    use exoplanets_catalog::common;
    use exoplanets_catalog::tables::commands;

    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about = None)]
    struct Cli {
        #[clap(subcommand)]
        command: Option<Commands>,
    }

    #[derive(Parser, Debug)]
    enum Commands {
        Serve,
        ViewFields { path: String }, // Print VOTable fields
        ViewSamples { 
            #[arg(short, long, default_value = "data/stellarhosts.vot")]
            path: String,
            #[arg(short, long, help = "Number of rows to show")]
            limit: Option<usize>,
            #[arg(short, long, help = "Category of columns to show (basic, position, stellar, photometry)")]
            category: Option<String>
        },
        ViewStats {
            #[arg(short, long, default_value = "data/stellarhosts.vot")]
            path: String
        },
        // Exoplanets commands
        ViewExoplanetsSamples { 
            #[arg(short, long, default_value = "data/exoplanets.vot")]
            path: String,
            #[arg(short, long, help = "Number of rows to show")]
            limit: Option<usize>,
            #[arg(short, long, help = "Category of columns to show (basic, discovery, orbital, physical)")]
            category: Option<String>
        },
        ViewExoplanetsStats {
            #[arg(short, long, default_value = "data/exoplanets.vot")]
            path: String
        }
    }

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Serve) | None => {
            start_server().await;
        }
        Some(Commands::ViewFields { path }) => {
            common::print_votable_headers(path);
        }
        Some(Commands::ViewSamples { path, limit, category }) => {
            let cat = category.as_ref().map(|s| s.as_str());
            if let Err(e) = commands::view_samples(path, *limit, cat) {
                eprintln!("Error viewing samples: {}", e);
            }
        }
        Some(Commands::ViewStats { path }) => {
            if let Err(e) = commands::view_stats(path) {
                eprintln!("Error viewing stats: {}", e);
            }
        }
        Some(Commands::ViewExoplanetsSamples { path, limit, category }) => {
            let cat = category.as_ref().map(|s| s.as_str());
            if let Err(e) = commands::view_exoplanets_samples(path, *limit, cat) {
                eprintln!("Error viewing exoplanets samples: {}", e);
            }
        }
        Some(Commands::ViewExoplanetsStats { path }) => {
            if let Err(e) = commands::view_exoplanets_stats(path) {
                eprintln!("Error viewing exoplanets stats: {}", e);
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
