use std::path::Path;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use exo_cli::{
    api::ApiClient,
    backend::{
        CatalogBackend, DatasetKind, RowsQuery, compiled_insight_meta,
        insight_meta_rows, resolve_backend, schema_rows,
    },
    commands, config, conversion, download,
    output::{self, OutputFormat},
    skill, votable_helpers,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, global = true, value_enum)]
    backend: Option<config::Backend>,
    #[arg(long, global = true)]
    api_base_url: Option<String>,
    #[arg(long, global = true)]
    data_dir: Option<String>,
    #[arg(short, long, global = true, value_enum)]
    output: Option<OutputFormat>,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Execute SQL against the selected backend
    Query {
        /// SQL query to execute (tables: stellarhosts, exoplanets)
        query: String,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Browse table rows from the selected backend
    Rows {
        #[arg(value_enum)]
        table: DatasetKind,
        #[arg(long, default_value_t = 1)]
        page: usize,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        sort: Option<String>,
        #[arg(long)]
        order: Option<String>,
        #[arg(long)]
        columns: Option<String>,
        #[arg(long)]
        filter: Option<String>,
    },
    /// View table schema from the selected backend
    Schema {
        #[arg(value_enum)]
        table: DatasetKind,
    },
    /// Download parquet and metadata files for offline use
    Download {
        #[arg(value_enum)]
        target: DownloadArg,
        #[arg(long)]
        directory: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Read or update persistent CLI config
    Config {
        #[clap(subcommand)]
        command: ConfigCommand,
    },
    /// Install the exodata agent skill
    Skill {
        #[clap(subcommand)]
        command: SkillCommand,
    },
    /// Run curated insight queries
    Insights {
        #[clap(subcommand)]
        command: InsightCommands,
    },
    /// Development and data preparation commands
    Dev {
        #[clap(subcommand)]
        command: DevCommands,
    },
}

#[derive(Parser, Debug)]
enum DevCommands {
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
    /// Execute SQL query against local parquet files
    Sql {
        /// SQL query to execute (tables: stellarhosts, exoplanets)
        query: String,
        #[arg(long, default_value = "data")]
        data_dir: String,
    },
    /// Development-only insight commands
    Insights {
        #[clap(subcommand)]
        command: DevInsightCommands,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum DownloadArg {
    Stellarhosts,
    Exoplanets,
    All,
}

impl From<DownloadArg> for download::DownloadTarget {
    fn from(value: DownloadArg) -> Self {
        match value {
            DownloadArg::Stellarhosts => Self::StellarHosts,
            DownloadArg::Exoplanets => Self::Exoplanets,
            DownloadArg::All => Self::All,
        }
    }
}

#[derive(Parser, Debug)]
enum ConfigCommand {
    /// Print the config file path
    Path,
    /// Print one config value
    Get { key: String },
    /// Set one config value
    Set { key: String, value: String },
}

#[derive(Parser, Debug)]
enum SkillCommand {
    /// Install the exodata agent skill locally or globally
    Install { scope: SkillInstallScope },
}

#[derive(Clone, Debug, ValueEnum)]
enum SkillInstallScope {
    Local,
    Global,
}

#[derive(Parser, Debug)]
enum InsightCommands {
    /// List available insight slugs and descriptions
    List,
    /// Run one insight query by slug
    #[command(
        after_help = "Examples:\n  exodata insights list\n  exodata insights run smallest-exoplanets-radius\n  exodata insights run nearest-stellar-hosts --data-dir data"
    )]
    Run {
        /// Insight slug to run. Use `exodata insights list` to see available slugs.
        slug: String,
    },
}

#[derive(Parser, Debug)]
enum DevInsightCommands {
    /// Run every insight query in registry order locally
    RunAll {
        /// Directory containing stellarhosts.parquet and exoplanets.parquet.
        #[arg(long, default_value = "data")]
        data_dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;
    let format = match cli.output {
        Some(format) => format,
        None => OutputFormat::from_config(&config.output.format)?,
    };

    match cli.command {
        Commands::Dev { command } => match command {
            DevCommands::ViewFields { path } => {
                votable_helpers::print_votable_headers(&path);
            }
            DevCommands::ViewSamples {
                path,
                limit,
                category,
            } => {
                commands::view_stellarhosts_samples(
                    Path::new(&path),
                    limit,
                    category.as_deref(),
                )?;
            }
            DevCommands::ViewStats { path } => {
                commands::view_stellarhosts_stats(&path)?;
            }
            DevCommands::ViewExoplanetsSamples {
                path,
                limit,
                category,
            } => {
                commands::view_exoplanets_samples(
                    &path,
                    limit,
                    category.as_deref(),
                )?;
            }
            DevCommands::ViewExoplanetsStats { path } => {
                commands::view_exoplanets_stats(&path)?;
            }
            DevCommands::ConvertRawFiles { data_dir } => {
                conversion::convert_raw_files(Path::new(&data_dir))?;
            }
            DevCommands::ViewMetadata { path, columns } => {
                commands::view_metadata(&path, columns.as_deref())?;
            }
            DevCommands::Sql { query, data_dir } => {
                commands::execute_sql(&query, &data_dir)?;
            }
            DevCommands::Insights { command } => match command {
                DevInsightCommands::RunAll { data_dir } => {
                    commands::run_all_insights(&data_dir)?;
                }
            },
        },
        Commands::Query { query, limit } => {
            let backend = resolve_backend(
                &config,
                cli.backend,
                cli.data_dir,
                cli.api_base_url,
            )?;
            let response = backend.sql(&query, limit)?;
            output::render_rows(&response.data, &response.columns, format)?;
        }
        Commands::Rows {
            table,
            page,
            limit,
            sort,
            order,
            columns,
            filter,
        } => {
            let backend = resolve_backend(
                &config,
                cli.backend,
                cli.data_dir,
                cli.api_base_url,
            )?;
            let response = backend.rows(RowsQuery {
                dataset: table,
                page,
                limit,
                sort_by: sort,
                order,
                columns,
                filter,
            })?;
            output::render_rows(&response.data, &response.columns, format)?;
        }
        Commands::Schema { table } => {
            let backend = resolve_backend(
                &config,
                cli.backend,
                cli.data_dir,
                cli.api_base_url,
            )?;
            let (rows, columns) = schema_rows(backend.schema(table)?);
            output::render_rows(&rows, &columns, format)?;
        }
        Commands::Download {
            target,
            directory,
            force,
        } => {
            let client = api_client(&config, cli.api_base_url)?;
            let directory = config.download_dir(directory);
            download::download(&client, target.into(), &directory, force)?;
        }
        Commands::Config { command } => match command {
            ConfigCommand::Path => {
                println!("{}", config::config_path()?.display())
            }
            ConfigCommand::Get { key } => {
                println!("{}", config::get_config_value(&config, &key)?);
            }
            ConfigCommand::Set { key, value } => {
                let mut config = config;
                config::set_config_value(&mut config, &key, &value)?;
                config.save()?;
            }
        },
        Commands::Skill { command } => match command {
            SkillCommand::Install { scope } => match scope {
                SkillInstallScope::Local => skill::install_local()?,
                SkillInstallScope::Global => skill::install_global()?,
            },
        },
        Commands::Insights { command } => match command {
            InsightCommands::List => {
                let meta = if config.backend(cli.backend) == config::Backend::Api
                {
                    let backend = resolve_backend(
                        &config,
                        cli.backend,
                        cli.data_dir,
                        cli.api_base_url,
                    )?;
                    backend.insights_list()?
                } else {
                    compiled_insight_meta()
                };
                let (rows, columns) = insight_meta_rows(meta);
                output::render_rows(&rows, &columns, format)?;
            }
            InsightCommands::Run { slug } => {
                let backend = resolve_backend(
                    &config,
                    cli.backend,
                    cli.data_dir,
                    cli.api_base_url,
                )?;
                let response = backend.insight_run(&slug)?;
                output::render_rows(&response.data, &response.columns, format)?;
            }
        },
    }

    Ok(())
}

fn api_client(
    config: &config::Config,
    base_url: Option<String>,
) -> Result<ApiClient> {
    ApiClient::new(config.api_base_url(base_url), config.api.timeout_seconds)
}
