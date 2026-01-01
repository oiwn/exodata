use clap::Parser;
use exo_core::tables::conversion;
use std::path::Path;

mod commands;

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
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::ViewFields { path } => {
            exo_core::common::print_votable_headers(&path);
        }
        Commands::ViewSamples {
            path,
            limit,
            category,
        } => {
            let cat = category.as_ref().map(|s| s.as_str());
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
            let cat = category.as_ref().map(|s| s.as_str());
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
    }
}
