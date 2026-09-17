use std::path::{Path, PathBuf};

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
use exo_prose::descriptions;

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
        /// Prompt used only to repair a failed validation (legacy alias: --style-prompt)
        #[arg(
            long,
            alias = "style-prompt",
            default_value = "content/stellarhost_repair_prompt.txt"
        )]
        repair_prompt: std::path::PathBuf,
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
        /// Generate or retry label-suffixed artifacts instead of served files
        #[arg(long)]
        label: Option<String>,
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
    /// Apply algorithmic normalizers to stored descriptions
    Normalize {
        #[arg(long, default_value = "content/systems")]
        content_dir: String,
        /// Normalize description_<label>.md instead of the served description.md
        #[arg(long)]
        label: Option<String>,
    },
    /// Analyze generated descriptions: stats, n-grams, tropes, metadata, anomalies
    Analyze {
        #[arg(long, global = true, default_value = "content/systems")]
        content_dir: String,
        /// Analyze description_<label>.md variant files instead of description.md
        #[arg(long, global = true)]
        label: Option<String>,
        #[clap(subcommand)]
        command: AnalyzeCommands,
    },
    /// Experiment: prepare and generate the fixed 20-system set into variant files
    Experiment {
        /// Variant label; outputs become description_<label>.md and never
        /// touch the served description.md
        #[arg(long)]
        label: String,
        #[arg(long, default_value = "content/systems")]
        input_dir: std::path::PathBuf,
        #[arg(long, default_value_t = 4, value_parser = clap::value_parser!(u32).range(1..))]
        concurrency: u32,
        #[arg(long, default_value_t = 1536, value_parser = clap::value_parser!(u32).range(1..))]
        max_tokens: u32,
        /// Regenerate even when the variant fingerprint matches
        #[arg(long)]
        force: bool,
    },
}

#[derive(Parser, Debug)]
enum AnalyzeCommands {
    /// Per-system text metrics: words, sentences, paragraphs, TTR, title
    Text,
    /// Corpus-level distributions of the per-system text metrics
    Summary,
    /// Word n-grams with corpus and document frequency
    Ngrams {
        /// Rows per n value
        #[arg(long, default_value_t = 25)]
        top: usize,
        /// Smallest n
        #[arg(long, default_value_t = 1)]
        min_n: usize,
        /// Largest n
        #[arg(long, default_value_t = 5)]
        max_n: usize,
        /// Count only the first n words of each sentence (monotony detector)
        #[arg(long)]
        openers: bool,
    },
    /// Sentence templates with numbers and units masked
    Templates {
        /// Maximum rows
        #[arg(long, default_value_t = 30)]
        top: usize,
        /// Minimum corpus frequency
        #[arg(long, default_value_t = 3)]
        min_count: u64,
    },
    /// Curated trope report: known prose warts with affected systems
    Tropes,
    /// Generation metadata aggregation: tokens, attempts, models, dates
    Metadata,
    /// Outliers, duplicate sentences, and numeric-density anomalies
    Anomalies {
        /// Minimum |z-score| for distribution outliers
        #[arg(long, default_value_t = 2.5)]
        z_threshold: f64,
        /// Maximum duplicate/repeat rows per kind
        #[arg(long, default_value_t = 30)]
        top: usize,
        /// Minimum systems sharing a duplicate sentence
        #[arg(long, default_value_t = 2)]
        min_duplicates: u64,
    },
    /// Compare two corpora (baseline snapshot vs candidate) on headline metrics
    Compare {
        /// Baseline content directory to compare against
        #[arg(long)]
        baseline_dir: String,
        /// Baseline variant label (baseline-dir reads description_<label>.md)
        #[arg(long)]
        baseline_label: Option<String>,
    },
    /// Write a machine-readable stats snapshot (JSON) for durable comparison
    Snapshot {
        /// Snapshot output path (suggested: content/stats/<name>.json)
        #[arg(long)]
        output_path: std::path::PathBuf,
    },
    /// Write a markdown report combining every analysis section
    Report {
        /// Report output path
        #[arg(long)]
        output_path: std::path::PathBuf,
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

/// Per-system commands default to compact single-line output when no
/// explicit `--output` is requested; table, json, and csv stay available.
fn render_per_system(
    rows: &[serde_json::Value],
    columns: &[String],
    requested: Option<OutputFormat>,
    line: impl Fn(&serde_json::Value) -> String,
) -> Result<()> {
    match requested.unwrap_or(OutputFormat::Lines) {
        OutputFormat::Lines => output::render_lines(rows, line),
        format => output::render_rows(rows, columns, format)?,
    }
    Ok(())
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
                    repair_prompt,
                    concurrency,
                    max_tokens,
                    force,
                    failed,
                    label,
                } => {
                    let rows = descriptions::batch::run(
                        &descriptions::batch::Options {
                            input_dir,
                            hostnames,
                            system_prompt,
                            repair_prompt,
                            concurrency: concurrency as usize,
                            max_tokens,
                            force,
                            failed,
                            label,
                        },
                    )?;
                    render_per_system(
                        &rows,
                        &descriptions::batch::columns(),
                        cli.output,
                        descriptions::batch::line,
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
                        render_per_system(
                            &rows,
                            &descriptions::prepare::columns(),
                            cli.output,
                            descriptions::prepare::line,
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
                    render_per_system(
                        &rows,
                        &descriptions::status::columns(),
                        cli.output,
                        descriptions::status::line,
                    )?;
                }
                DescriptionCommands::Normalize { content_dir, label } => {
                    let rows = descriptions::normalize::run(
                        Path::new(&content_dir),
                        label.as_deref(),
                    )?;
                    render_per_system(
                        &rows,
                        &descriptions::normalize::columns(),
                        cli.output,
                        |row| {
                            let hostname =
                                row["hostname"].as_str().unwrap_or("?");
                            let path = row["path"].as_str().unwrap_or_default();
                            let changed = if row["changed"] == true {
                                "rewritten"
                            } else {
                                "unchanged"
                            };
                            format!("{hostname:<26} {changed:<10} {path}")
                        },
                    )?;
                }
                DescriptionCommands::Analyze {
                    content_dir,
                    label,
                    command,
                } => {
                    use descriptions::analyze;
                    let corpus = analyze::Corpus::load(
                        Path::new(&content_dir),
                        label.as_deref(),
                    )?;
                    match command {
                        AnalyzeCommands::Text => {
                            render_per_system(
                                &analyze::text::run(&corpus),
                                &analyze::text::columns(),
                                cli.output,
                                analyze::text::line,
                            )?;
                        }
                        AnalyzeCommands::Summary => {
                            render_per_system(
                                &analyze::text::summary(&corpus),
                                &analyze::text::summary_columns(),
                                cli.output,
                                analyze::text::summary_line,
                            )?;
                        }
                        AnalyzeCommands::Ngrams {
                            top,
                            min_n,
                            max_n,
                            openers,
                        } => {
                            render_per_system(
                                &analyze::ngrams::run(
                                    &corpus,
                                    &analyze::ngrams::Options {
                                        top,
                                        min_n,
                                        max_n,
                                        openers,
                                    },
                                ),
                                &analyze::ngrams::columns(),
                                cli.output,
                                analyze::ngrams::line,
                            )?;
                        }
                        AnalyzeCommands::Templates { top, min_count } => {
                            render_per_system(
                                &analyze::ngrams::templates(
                                    &corpus,
                                    &analyze::ngrams::TemplateOptions {
                                        top,
                                        min_count,
                                    },
                                ),
                                &analyze::ngrams::template_columns(),
                                cli.output,
                                analyze::ngrams::template_line,
                            )?;
                        }
                        AnalyzeCommands::Tropes => {
                            render_per_system(
                                &analyze::tropes::run(&corpus),
                                &analyze::tropes::columns(),
                                cli.output,
                                analyze::tropes::line,
                            )?;
                        }
                        AnalyzeCommands::Metadata => {
                            render_per_system(
                                &analyze::meta::run(&corpus),
                                &analyze::meta::columns(),
                                cli.output,
                                analyze::meta::line,
                            )?;
                        }
                        AnalyzeCommands::Anomalies {
                            z_threshold,
                            top,
                            min_duplicates,
                        } => {
                            render_per_system(
                                &analyze::anomalies::run(
                                    &corpus,
                                    &analyze::anomalies::Options {
                                        z_threshold,
                                        top,
                                        min_duplicate_systems: min_duplicates,
                                    },
                                ),
                                &analyze::anomalies::columns(),
                                cli.output,
                                analyze::anomalies::line,
                            )?;
                        }
                        AnalyzeCommands::Compare {
                            baseline_dir,
                            baseline_label,
                        } => {
                            let baseline = analyze::Corpus::load(
                                Path::new(&baseline_dir),
                                baseline_label.as_deref(),
                            )?;
                            render_per_system(
                                &analyze::compare::run(&baseline, &corpus),
                                &analyze::compare::columns(),
                                cli.output,
                                analyze::compare::line,
                            )?;
                        }
                        AnalyzeCommands::Snapshot { output_path } => {
                            render_per_system(
                                &analyze::snapshot::run(
                                    &corpus,
                                    Path::new(&content_dir),
                                    &output_path,
                                )?,
                                &analyze::snapshot::columns(),
                                cli.output,
                                analyze::snapshot::line,
                            )?;
                        }
                        AnalyzeCommands::Report { output_path } => {
                            render_per_system(
                                &analyze::report::run(&corpus, &output_path)?,
                                &analyze::report::columns(),
                                cli.output,
                                analyze::report::line,
                            )?;
                        }
                    }
                }
                DescriptionCommands::Experiment {
                    label,
                    input_dir,
                    concurrency,
                    max_tokens,
                    force,
                } => {
                    let rows = descriptions::experiment::run(
                        &descriptions::experiment::Options {
                            data_dir: PathBuf::from(
                                cli.data_dir.as_deref().unwrap_or("data"),
                            ),
                            input_dir,
                            label,
                            force,
                            concurrency: concurrency as usize,
                            max_tokens,
                        },
                    )?;
                    render_per_system(
                        &rows,
                        &descriptions::experiment::columns(),
                        cli.output,
                        descriptions::experiment::line,
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
