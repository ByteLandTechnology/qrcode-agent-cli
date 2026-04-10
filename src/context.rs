use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::RenderMode;

pub const APP_NAME: &str = "qrcode-agent-cli";
pub const DEFAULT_IMAGE_SIZE: u32 = 256;

const CONFIG_DIR_ENV: &str = "QRCODE_AGENT_CLI_CONFIG_DIR";
const DATA_DIR_ENV: &str = "QRCODE_AGENT_CLI_DATA_DIR";
const STATE_DIR_ENV: &str = "QRCODE_AGENT_CLI_STATE_DIR";
const CACHE_DIR_ENV: &str = "QRCODE_AGENT_CLI_CACHE_DIR";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_render: Option<RenderMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_image_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePaths {
    pub config_dir: String,
    pub data_dir: String,
    pub state_dir: String,
    pub cache_dir: String,
    pub context_file: String,
}

impl RuntimePaths {
    pub fn config_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.config_dir)
    }

    pub fn context_file_path(&self) -> PathBuf {
        PathBuf::from(&self.context_file)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeEnvOverrides {
    pub config_dir: &'static str,
    pub data_dir: &'static str,
    pub state_dir: &'static str,
    pub cache_dir: &'static str,
}

pub fn runtime_env_overrides() -> RuntimeEnvOverrides {
    RuntimeEnvOverrides {
        config_dir: CONFIG_DIR_ENV,
        data_dir: DATA_DIR_ENV,
        state_dir: STATE_DIR_ENV,
        cache_dir: CACHE_DIR_ENV,
    }
}

pub fn runtime_paths() -> Result<RuntimePaths> {
    let config_dir = env_path(CONFIG_DIR_ENV)
        .or_else(|| dirs::config_dir().map(|path| path.join(APP_NAME)))
        .ok_or_else(|| anyhow!("Unable to resolve a user-scoped config directory"))?;
    let data_dir = env_path(DATA_DIR_ENV)
        .or_else(|| dirs::data_dir().map(|path| path.join(APP_NAME)))
        .ok_or_else(|| anyhow!("Unable to resolve a user-scoped data directory"))?;
    let state_dir = env_path(STATE_DIR_ENV)
        .or_else(|| dirs::state_dir().map(|path| path.join(APP_NAME)))
        .or_else(|| dirs::data_local_dir().map(|path| path.join(APP_NAME).join("state")))
        .ok_or_else(|| anyhow!("Unable to resolve a user-scoped state directory"))?;
    let cache_dir = env_path(CACHE_DIR_ENV)
        .or_else(|| dirs::cache_dir().map(|path| path.join(APP_NAME)))
        .ok_or_else(|| anyhow!("Unable to resolve a user-scoped cache directory"))?;
    let context_file = config_dir.join("active-context.toml");

    Ok(RuntimePaths {
        config_dir: config_dir.display().to_string(),
        data_dir: data_dir.display().to_string(),
        state_dir: state_dir.display().to_string(),
        cache_dir: cache_dir.display().to_string(),
        context_file: context_file.display().to_string(),
    })
}

pub fn load_active_context() -> Result<ActiveContext> {
    let paths = runtime_paths()?;
    let context_file = paths.context_file_path();
    if !context_file.exists() {
        return Ok(ActiveContext::default());
    }

    let raw = fs::read_to_string(&context_file)
        .with_context(|| format!("Failed to read {}", context_file.display()))?;
    toml::from_str(&raw).with_context(|| {
        format!(
            "Failed to parse persisted context from {}",
            context_file.display()
        )
    })
}

pub fn save_active_context(context: &ActiveContext) -> Result<()> {
    let paths = runtime_paths()?;
    let config_dir = paths.config_dir_path();
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("Failed to create {}", config_dir.display()))?;

    let serialized =
        toml::to_string_pretty(context).context("Failed to serialize the active context")?;
    fs::write(paths.context_file_path(), serialized)
        .with_context(|| format!("Failed to write {}", paths.context_file_path().display()))?;
    Ok(())
}

pub fn clear_active_context() -> Result<bool> {
    let paths = runtime_paths()?;
    let context_file = paths.context_file_path();
    if !context_file.exists() {
        return Ok(false);
    }

    fs::remove_file(&context_file)
        .with_context(|| format!("Failed to remove {}", context_file.display()))?;
    Ok(true)
}

pub fn default_output_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()
        .context("Failed to resolve the current working directory")?
        .join("qrcode.png"))
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
