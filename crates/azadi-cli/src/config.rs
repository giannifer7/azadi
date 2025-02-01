// crates/azadi-cli/src/config.rs
use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize, Default)]
pub struct AzadiConfig {
    // File patterns and locations
    #[serde(default)]
    pub input_dir: Option<PathBuf>,
    #[serde(default)]
    pub output_dir: Option<PathBuf>,
    #[serde(default)]
    pub work_dir: Option<PathBuf>,

    // Syntax configuration
    #[serde(default)]
    pub special: Option<char>,
    #[serde(default)]
    pub open_delim: Option<String>,
    #[serde(default)]
    pub close_delim: Option<String>,
    #[serde(default)]
    pub chunk_end: Option<String>,
    #[serde(default)]
    pub comment_markers: Option<String>,

    // Path handling
    #[serde(default)]
    pub include: Option<String>,
    #[serde(default)]
    pub pathsep: Option<String>,

    // Feature flags
    #[serde(default)]
    pub pydef: Option<bool>,
    #[serde(default)]
    pub save_macro: Option<bool>,
    #[serde(default)]
    pub dump_ast: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PyProject {
    #[serde(default)]
    tool: PyProjectTool,
}

#[derive(Debug, Deserialize, Default)]
struct PyProjectTool {
    #[serde(default)]
    azadi: AzadiConfig,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    #[serde(default)]
    package: CargoPackage,
}

#[derive(Debug, Deserialize, Default)]
struct CargoPackage {
    #[serde(default)]
    metadata: CargoMetadata,
}

#[derive(Debug, Deserialize, Default)]
struct CargoMetadata {
    #[serde(default)]
    azadi: AzadiConfig,
}

impl AzadiConfig {
    pub fn from_file(path: &Path) -> Result<Option<Self>, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(Some(toml::from_str(&content)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ConfigError::Io(e)),
        }
    }

    pub fn from_nearest() -> Result<Self, ConfigError> {
        // Start with an empty config
        let mut config = AzadiConfig::default();

        // Try Cargo.toml first (least specific)
        if let Some(cargo_config) = Self::from_cargo_toml()? {
            config.merge(cargo_config);
        }

        // Then pyproject.toml
        if let Some(py_config) = Self::from_pyproject_toml()? {
            config.merge(py_config);
        }

        // Finally azadi.toml (most specific)
        if let Some(azadi_config) = Self::from_file(Path::new("azadi.toml"))? {
            config.merge(azadi_config);
        }

        Ok(config)
    }

    fn from_pyproject_toml() -> Result<Option<Self>, ConfigError> {
        if let Some(content) = read_if_exists("pyproject.toml")? {
            let pyproject: PyProject = toml::from_str(&content)?;
            Ok(Some(pyproject.tool.azadi))
        } else {
            Ok(None)
        }
    }

    fn from_cargo_toml() -> Result<Option<Self>, ConfigError> {
        if let Some(content) = read_if_exists("Cargo.toml")? {
            let cargo: CargoToml = toml::from_str(&content)?;
            Ok(Some(cargo.package.metadata.azadi))
        } else {
            Ok(None)
        }
    }

    /// Merge another config into this one, taking values from other if they are Some
    fn merge(&mut self, other: Self) {
        macro_rules! merge_field {
            ($field:ident) => {
                if let Some(value) = other.$field {
                    self.$field = Some(value);
                }
            };
        }

        merge_field!(input_dir);
        merge_field!(output_dir);
        merge_field!(work_dir);
        merge_field!(special);
        merge_field!(open_delim);
        merge_field!(close_delim);
        merge_field!(chunk_end);
        merge_field!(comment_markers);
        merge_field!(include);
        merge_field!(pathsep);
        merge_field!(pydef);
        merge_field!(save_macro);
        merge_field!(dump_ast);
    }
}

fn read_if_exists(path: &str) -> Result<Option<String>, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
