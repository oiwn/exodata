use clap::Parser;
use exo_cli::{commands, conversion, votable_helpers};
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// View fields from a VOTable file
    ViewFields { path: String },
    /// View samples from stellarhosts parquet file
    ViewSamples {
        #[arg(short, long, default_value = "data/stellarhosts.parquet")]
        path: String,
        #[arg(short, long, help = "Number of rows to show")]
        limit: Option<usize>,
        #[arg(
            short,
            long,
            help = "Category of columns to show (basic, position, stellar, photometry)"
        )]
        category: Option<String>,
    },
    /// View statistics from stellarhosts parquet file
    ViewStats {
        #[arg(short, long, default_value = "data/stellarhosts.parquet")]
        path: String,
    },
    /// View samples from exoplanets parquet file
    ViewExoplanetsSamples {
        #[arg(short, long, default_value = "data/exoplanets.parquet")]
        path: String,
        #[arg(short, long, help = "Number of rows to show")]
        limit: Option<usize>,
        #[arg(
            short,
            long,
            help = "Category of columns to show (basic, discovery, orbital, physical)"
        )]
        category: Option<String>,
    },
    /// View statistics from exoplanets parquet file
    ViewExoplanetsStats {
        #[arg(short, long, default_value = "data/exoplanets.parquet")]
        path: String,
    },
    /// Convert all .vot files in the data directory to parquet
    #[clap(name = "convert-raw-files")]
    ConvertRawFiles {
        #[arg(short, long, default_value = "data")]
        data_dir: String,
    },
    /// View column metadata from a VOTable file
    ViewMetadata {
        #[arg(short, long, default_value = "data/exoplanets.vot")]
        path: String,
        #[arg(
            short,
            long,
            help = "Filter to specific columns (comma-separated)"
        )]
        columns: Option<String>,
    },
    /// Execute SQL query against parquet files
    Sql {
        /// SQL query to execute (tables: stellarhosts, exoplanets)
        query: String,
        #[arg(long, default_value = "data")]
        data_dir: String,
    },
    /// Run curated insight queries
    Insights {
        #[clap(subcommand)]
        command: InsightCommands,
    },
}

#[derive(Parser, Debug)]
enum InsightCommands {
    /// List available insight slugs and descriptions
    List,
    /// Run one insight query by slug
    #[command(
        after_help = "Examples:\n  exo insights list\n  exo insights run smallest-exoplanets-radius\n  exo insights run nearest-stellar-hosts --data-dir data"
    )]
    Run {
        /// Insight slug to run. Use `exo insights list` to see available slugs.
        slug: String,
        /// Directory containing stellarhosts.parquet and exoplanets.parquet.
        #[arg(long, default_value = "data")]
        data_dir: String,
    },
    /// Run every insight query in registry order
    RunAll {
        /// Directory containing stellarhosts.parquet and exoplanets.parquet.
        #[arg(long, default_value = "data")]
        data_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::ViewFields { path } => {
            votable_helpers::print_votable_headers(&path);
        }
        Commands::ViewSamples {
            path,
            limit,
            category,
        } => {
            let cat = category.as_deref();
            if let Err(e) =
                commands::view_stellarhosts_samples(Path::new(&path), limit, cat)
            {
                eprintln!("Error viewing samples: {}", e);
            }
        }
        Commands::ViewStats { path } => {
            if let Err(e) = commands::view_stellarhosts_stats(&path) {
                eprintln!("Error viewing stats: {}", e);
            }
        }
        Commands::ViewExoplanetsSamples {
            path,
            limit,
            category,
        } => {
            let cat = category.as_deref();
            if let Err(e) = commands::view_exoplanets_samples(&path, limit, cat) {
                eprintln!("Error viewing exoplanets samples: {}", e);
            }
        }
        Commands::ViewExoplanetsStats { path } => {
            if let Err(e) = commands::view_exoplanets_stats(&path) {
                eprintln!("Error viewing exoplanets stats: {}", e);
            }
        }
        Commands::ConvertRawFiles { data_dir } => {
            if let Err(e) = conversion::convert_raw_files(Path::new(&data_dir)) {
                eprintln!("Error converting VOTable files: {}", e);
            }
        }
        Commands::ViewMetadata { path, columns } => {
            if let Err(e) = commands::view_metadata(&path, columns.as_deref()) {
                eprintln!("Error viewing metadata: {}", e);
            }
        }
        Commands::Sql { query, data_dir } => {
            if let Err(e) = commands::execute_sql(&query, &data_dir) {
                eprintln!("Error executing SQL: {}", e);
            }
        }
        Commands::Insights { command } => match command {
            InsightCommands::List => {
                commands::list_insights();
            }
            InsightCommands::Run { slug, data_dir } => {
                if let Err(e) = commands::run_insight(&slug, &data_dir) {
                    eprintln!("Error running insight: {}", e);
                }
            }
            InsightCommands::RunAll { data_dir } => {
                if let Err(e) = commands::run_all_insights(&data_dir) {
                    eprintln!("Error running insights: {}", e);
                }
            }
        },
    }
}
