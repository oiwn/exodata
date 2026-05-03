use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Auto,
    Api,
    Local,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Auto => write!(f, "auto"),
            Backend::Api => write!(f, "api"),
            Backend::Local => write!(f, "local"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_backend")]
    pub default_backend: Backend,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub local: LocalConfig,
    #[serde(default)]
    pub downloads: DownloadConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApiConfig {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalConfig {
    #[serde(default = "default_data_dir_string")]
    pub data_dir: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DownloadConfig {
    #[serde(default = "default_data_dir_string")]
    pub directory: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_format")]
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_backend: default_backend(),
            api: ApiConfig::default(),
            local: LocalConfig::default(),
            downloads: DownloadConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            timeout_seconds: default_timeout_seconds(),
        }
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir_string(),
        }
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            directory: default_data_dir_string(),
            overwrite: false,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_output_format(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        toml::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create {}", parent.display())
            })?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(&path, text)
            .with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn backend(&self, flag: Option<Backend>) -> Backend {
        if let Some(backend) = flag {
            return backend;
        }
        if let Ok(value) = env::var("EXO_BACKEND") {
            match value.trim().to_ascii_lowercase().as_str() {
                "auto" => return Backend::Auto,
                "api" => return Backend::Api,
                "local" => return Backend::Local,
                _ => {}
            }
        }
        self.default_backend
    }

    pub fn api_base_url(&self, flag: Option<String>) -> String {
        flag.or_else(|| env::var("EXO_API_BASE_URL").ok())
            .unwrap_or_else(|| self.api.base_url.clone())
            .trim_end_matches('/')
            .to_string()
    }

    pub fn data_dir(&self, flag: Option<String>) -> String {
        flag.or_else(|| env::var("EXO_DATA_DIR").ok())
            .unwrap_or_else(|| self.local.data_dir.clone())
    }

    pub fn download_dir(&self, flag: Option<String>) -> String {
        flag.or_else(|| env::var("EXO_DOWNLOAD_DIR").ok())
            .unwrap_or_else(|| self.downloads.directory.clone())
    }
}

pub fn config_path() -> Result<PathBuf> {
    Ok(base_dir()?.join("config.toml"))
}

pub fn base_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".exodata"))
        .ok_or_else(|| anyhow!("could not determine home directory"))
}

pub fn get_config_value(config: &Config, key: &str) -> Result<String> {
    match key {
        "default_backend" => Ok(config.default_backend.to_string()),
        "api.base_url" => Ok(config.api.base_url.clone()),
        "api.timeout_seconds" => Ok(config.api.timeout_seconds.to_string()),
        "local.data_dir" => Ok(config.local.data_dir.clone()),
        "downloads.directory" => Ok(config.downloads.directory.clone()),
        "downloads.overwrite" => Ok(config.downloads.overwrite.to_string()),
        "output.format" => Ok(config.output.format.clone()),
        _ => Err(anyhow!("unknown config key '{}'", key)),
    }
}

pub fn set_config_value(
    config: &mut Config,
    key: &str,
    value: &str,
) -> Result<()> {
    match key {
        "default_backend" => {
            config.default_backend = parse_backend(value)?;
        }
        "api.base_url" => {
            config.api.base_url = value.trim_end_matches('/').to_string();
        }
        "api.timeout_seconds" => {
            config.api.timeout_seconds = value.parse()?;
        }
        "local.data_dir" => {
            config.local.data_dir = value.to_string();
        }
        "downloads.directory" => {
            config.downloads.directory = value.to_string();
        }
        "downloads.overwrite" => {
            config.downloads.overwrite = value.parse()?;
        }
        "output.format" => {
            config.output.format = value.to_string();
        }
        _ => return Err(anyhow!("unknown config key '{}'", key)),
    }
    Ok(())
}

fn parse_backend(value: &str) -> Result<Backend> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(Backend::Auto),
        "api" => Ok(Backend::Api),
        "local" => Ok(Backend::Local),
        _ => Err(anyhow!("backend must be 'auto', 'api', or 'local'")),
    }
}

fn default_backend() -> Backend {
    Backend::Auto
}

fn default_base_url() -> String {
    "https://exodata.space".to_string()
}

fn default_timeout_seconds() -> u64 {
    30
}

fn default_data_dir_string() -> String {
    base_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "~/.exodata".to_string())
}

fn default_output_format() -> String {
    "table".to_string()
}
