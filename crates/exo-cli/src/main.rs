use std::path::Path;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use exo_cli::{
    api::ApiClient,
    backend::{
        CatalogBackend, DatasetKind, RowsQuery, compiled_insight_meta,
        insight_meta_rows, resolve_backend, schema_rows,
    },
    commands, config, conversion, descriptions, download,
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
    /// Inspect generated descriptions using local source dates
    Descriptions {
        #[clap(subcommand)]
        command: DescriptionCommands,
    },
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

#[derive(Parser, Debug)]
enum DescriptionCommands {
    /// Generate changed saved requests concurrently and persist per-system results
    GenerateBatch {
        #[arg(long, default_value = "content/systems")]
        input_dir: std::path::PathBuf,
        /// Optional repeated exact hostname filters
        #[arg(long = "hostname")]
        hostnames: Vec<String>,
        #[arg(long, default_value = "content/stellarhost_prompt.txt")]
        system_prompt: std::path::PathBuf,
        /// Editorial second-stage system prompt
        #[arg(long, default_value = "content/stellarhost_editor_prompt.txt")]
        editor_prompt: std::path::PathBuf,
        /// Verifier third-stage system prompt
        #[arg(long, default_value = "content/stellarhost_critic_prompt.txt")]
        critic_prompt: std::path::PathBuf,
        #[arg(long, default_value_t = 4, value_parser = clap::value_parser!(u32).range(1..))]
        concurrency: u32,
        #[arg(long, default_value_t = 1536, value_parser = clap::value_parser!(u32).range(1..))]
        max_tokens: u32,
        /// Regenerate even when successful input fingerprints match
        #[arg(long)]
        force: bool,
        /// Retry only systems whose latest attempt failed (fail.toml present)
        #[arg(long)]
        failed: bool,
    },
    /// Prepare offline stellar-host evidence and a writing request
    Prepare {
        /// Exact NASA hostnames; repeat to prepare several systems in one run
        #[arg(
            long = "hostname",
            required_unless_present = "all",
            conflicts_with = "all"
        )]
        hostnames: Vec<String>,
        #[arg(long, default_value = "content/systems")]
        output_dir: std::path::PathBuf,
        /// Replace preparation files, preserving articles and generation metadata
        #[arg(long)]
        force: bool,
        /// Prepare every hostname that has planet rows in the local dataset
        #[arg(long)]
        all: bool,
        /// Select and validate every system without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Probe DeepSeek with one request capped at 32 output tokens
    Probe,
    /// Generate text from an arbitrary UTF-8 input file using DeepSeek
    Generate {
        /// File sent verbatim as the user message (TOML or plain text)
        #[arg(long)]
        input: std::path::PathBuf,
        /// Optional UTF-8 file sent as a separate system message
        #[arg(long)]
        system_prompt: Option<std::path::PathBuf>,
        /// Maximum output tokens for this single request
        #[arg(long, default_value_t = 256, value_parser = clap::value_parser!(u32).range(1..))]
        max_tokens: u32,
    },
    /// Report systems needing descriptions or regeneration
    Scan {
        #[arg(long)]
        all: bool,
        #[arg(long, default_value = "content/systems")]
        content_dir: String,
    },
    /// Summarize per-system generation state, usage, and failures
    Status {
        #[arg(long, default_value = "content/systems")]
        content_dir: String,
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
            DevCommands::Descriptions { command } => match command {
                DescriptionCommands::GenerateBatch {
                    input_dir,
                    hostnames,
                    system_prompt,
                    editor_prompt,
                    critic_prompt,
                    concurrency,
                    max_tokens,
                    force,
                    failed,
                } => {
                    let rows = descriptions::batch::run(
                        &descriptions::batch::Options {
                            input_dir,
                            hostnames,
                            system_prompt,
                            editor_prompt,
                            critic_prompt,
                            concurrency: concurrency as usize,
                            max_tokens,
                            force,
                            failed,
                        },
                    )?;
                    output::render_rows(
                        &rows,
                        &descriptions::batch::columns(),
                        format,
                    )?;
                    if rows.iter().any(|row| {
                        row["status"] == "failed"
                            || row["status"] == "not_started"
                    }) {
                        anyhow::bail!(
                            "Batch incomplete; see per-system outcomes and fail.toml files"
                        );
                    }
                }
                DescriptionCommands::Prepare {
                    hostnames,
                    output_dir,
                    force,
                    all,
                    dry_run,
                } => {
                    let catalog = descriptions::prepare::Catalog::load(
                        Path::new(cli.data_dir.as_deref().unwrap_or("data")),
                    )?;
                    let hostnames = if all {
                        catalog.all_hostnames()?
                    } else {
                        hostnames
                    };
                    let mut rows = Vec::new();
                    let mut failed = false;
                    for hostname in &hostnames {
                        eprintln!("Preparing {hostname}");
                        match catalog.prepare(
                            &output_dir,
                            hostname,
                            force,
                            dry_run,
                        ) {
                            Ok(row) => {
                                if let Some(list) = row["diagnostics"].as_array()
                                {
                                    for diagnostic in list {
                                        if let Some(text) = diagnostic.as_str() {
                                            eprintln!("  - {text}");
                                        }
                                    }
                                }
                                let count = row["diagnostics_count"]
                                    .as_u64()
                                    .unwrap_or_default();
                                eprintln!(
                                    "Prepared {hostname} ({count} diagnostic{})",
                                    if count == 1 { "" } else { "s" }
                                );
                                rows.push(row);
                            }
                            Err(error) => {
                                failed = true;
                                let message = format!("{hostname}: {error:#}");
                                if format == OutputFormat::Json {
                                    rows.push(serde_json::json!({
                                        "hostname": hostname, "error": message,
                                    }));
                                } else {
                                    eprintln!("{message}");
                                }
                            }
                        }
                    }
                    if !rows.is_empty() {
                        output::render_rows(
                            &rows,
                            &descriptions::prepare::columns(),
                            format,
                        )?;
                    }
                    if failed {
                        anyhow::bail!(
                            "Preparation failed for at least one hostname"
                        );
                    }
                }
                DescriptionCommands::Probe => {
                    let row = descriptions::probe::run()?;
                    output::render_rows(
                        &[row],
                        &descriptions::probe::columns(),
                        format,
                    )?;
                }
                DescriptionCommands::Generate {
                    input,
                    system_prompt,
                    max_tokens,
                } => {
                    let text =
                        std::fs::read_to_string(&input).map_err(|error| {
                            anyhow::anyhow!(
                                "Cannot read input file {}: {error}",
                                input.display()
                            )
                        })?;
                    let system_text = system_prompt
                        .as_ref()
                        .map(|path| {
                            std::fs::read_to_string(path).map_err(|error| {
                                anyhow::anyhow!(
                                    "Cannot read system prompt {}: {error}",
                                    path.display()
                                )
                            })
                        })
                        .transpose()?;
                    let row = descriptions::probe::generate_with_system(
                        &text,
                        system_text.as_deref(),
                        max_tokens,
                    )?;
                    output::render_rows(
                        &[row],
                        &descriptions::probe::columns(),
                        format,
                    )?;
                }
                DescriptionCommands::Scan { all, content_dir } => {
                    let rows = descriptions::scan(
                        Path::new(cli.data_dir.as_deref().unwrap_or("data")),
                        Path::new(&content_dir),
                        all,
                    )?;
                    output::render_rows(&rows, &descriptions::columns(), format)?;
                }
                DescriptionCommands::Status { content_dir } => {
                    let rows =
                        descriptions::status::run(Path::new(&content_dir))?;
                    output::render_rows(
                        &rows,
                        &descriptions::status::columns(),
                        format,
                    )?;
                }
            },
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
