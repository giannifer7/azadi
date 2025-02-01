// crates/azadi-cli/src/options.rs

use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Configuration that can be loaded from files
#[derive(Debug, Deserialize, Default)]
pub struct FileConfig {
    // File patterns and locations
    pub input_dir: Option<String>,
    pub output_dir: Option<String>,
    pub work_dir: Option<String>,

    // Syntax configuration
    pub special: Option<char>,
    pub open_delim: Option<String>,
    pub close_delim: Option<String>,
    pub chunk_end: Option<String>,
    pub comment_markers: Option<String>,

    // Path handling
    pub include: Option<String>,
    pub pathsep: Option<String>,

    // Feature flags
    pub pydef: Option<bool>,
    pub save_macro: Option<bool>,
    pub dump_ast: Option<bool>,
}

#[derive(Parser, Debug)]
#[command(
    name = "azadi",
    about = "Process files through macro expansion and literate programming"
)]
pub struct Args {
    /// Input files to process (use "-" for stdin)
    #[arg(
        required = true,
        help = "One or more input files to process. Use - for stdin"
    )]
    pub files: Vec<PathBuf>,

    /// Base directory for input files (ignored for stdin)
    #[arg(
        long = "input-dir",
        help = "Base directory for resolving input files (ignored when reading from stdin)"
    )]
    pub input_dir: Option<String>,

    /// Output directory for generated files
    #[arg(long, help = "Directory for output files")]
    pub output_dir: Option<String>,

    /// Special character used to mark macro invocations
    #[arg(long, help = "Special character for macro syntax")]
    pub special: Option<char>,

    /// Working directory for temporary and backup files
    #[arg(long, help = "Directory for temporary and backup files")]
    pub work_dir: Option<String>,

    /// Keep intermediate files from macro processing
    #[arg(long, help = "Save intermediate macro output files")]
    pub save_macro: bool,

    /// Only perform macro processing
    #[arg(long, help = "Stop after macro processing")]
    pub macro_only: bool,

    /// Skip macro processing and only run noweb
    #[arg(long, help = "Skip macro processing")]
    pub noweb_only: bool,

    /// Colon-separated list of include paths for macro processing
    #[arg(long, help = "Pathsep-separated list of include paths")]
    pub include: Option<String>,

    /// Path separator for include paths
    #[arg(
        long = "pathsep",
        help = "Path separator character (; on Windows, : on Unix-like systems)"
    )]
    pub pathsep: Option<String>,

    /// Opening delimiter for chunk definitions
    #[arg(long, help = "Opening delimiter for chunk definitions")]
    pub open_delim: Option<String>,

    /// Closing delimiter for chunk definitions
    #[arg(long, help = "Closing delimiter for chunk definitions")]
    pub close_delim: Option<String>,

    /// End marker for chunk definitions
    #[arg(long, help = "Marker that ends a chunk definition")]
    pub chunk_end: Option<String>,

    /// Comment markers used in source files
    #[arg(long, help = "Comma-separated list of comment markers")]
    pub comment_markers: Option<String>,

    /// Specific chunks to extract
    #[arg(long, help = "Comma-separated list of chunks to extract")]
    pub chunks: Option<String>,

    /// Enable Python macro definitions
    #[arg(long = "pydef", help = "Enable Python macro definitions")]
    pub pydef: bool,

    /// Parse input and emit AST in JSONL format
    #[arg(long = "dump_ast", help = "Parse input and emit AST in JSONL format")]
    pub dump_ast: bool,

    /// Config file path
    #[arg(long, help = "Path to config file")]
    config: Option<PathBuf>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            files: vec![],
            input_dir: Some(".".into()),
            output_dir: Some("gen".into()),
            special: Some('%'),
            work_dir: Some("_azadi_work".into()),
            save_macro: false,
            macro_only: false,
            noweb_only: false,
            include: Some(".".into()),
            pathsep: Some(if cfg!(windows) {
                ";".into()
            } else {
                ":".into()
            }),
            open_delim: Some("<[".into()),
            close_delim: Some("]>".into()),
            chunk_end: Some("$$".into()),
            comment_markers: Some("#,//".into()),
            chunks: None,
            pydef: false,
            dump_ast: false,
            config: None,
        }
    }
}

impl FileConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Debug)]
pub struct Options {
    pub files: Vec<PathBuf>,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub special: char,
    pub work_dir: PathBuf,
    pub save_macro: bool,
    pub macro_only: bool,
    pub noweb_only: bool,
    pub include: String,
    pub pathsep: String,
    pub open_delim: String,
    pub close_delim: String,
    pub chunk_end: String,
    pub comment_markers: String,
    pub chunks: Option<String>,
    pub pydef: bool,
    pub dump_ast: bool,
}

impl Options {
    pub fn from_args_and_config(mut args: Args) -> Result<Self, ConfigError> {
        // Try to load config file if specified
        let file_config = if let Some(config_path) = args.config.as_ref() {
            Some(FileConfig::from_file(config_path)?)
        } else {
            // Try default locations
            for path in ["azadi.toml", "pyproject.toml", "Cargo.toml"] {
                if let Ok(config) = FileConfig::from_file(&PathBuf::from(path)) {
                    break Some(config);
                }
            }
            None
        };

        // Command line args take precedence over config file
        Ok(Self {
            files: args.files,
            input_dir: PathBuf::from(
                args.input_dir
                    .or(file_config.as_ref().and_then(|c| c.input_dir.clone()))
                    .unwrap_or_else(|| ".".into()),
            ),
            output_dir: PathBuf::from(
                args.output_dir
                    .or(file_config.as_ref().and_then(|c| c.output_dir.clone()))
                    .unwrap_or_else(|| "gen".into()),
            ),
            special: args
                .special
                .or(file_config.as_ref().and_then(|c| c.special))
                .unwrap_or('%'),
            work_dir: PathBuf::from(
                args.work_dir
                    .or(file_config.as_ref().and_then(|c| c.work_dir.clone()))
                    .unwrap_or_else(|| "_azadi_work".into()),
            ),
            save_macro: args.save_macro
                || file_config
                    .as_ref()
                    .and_then(|c| c.save_macro)
                    .unwrap_or(false),
            macro_only: args.macro_only,
            noweb_only: args.noweb_only,
            include: args
                .include
                .or(file_config.as_ref().and_then(|c| c.include.clone()))
                .unwrap_or_else(|| ".".into()),
            pathsep: args
                .pathsep
                .or(file_config.as_ref().and_then(|c| c.pathsep.clone()))
                .unwrap_or_else(|| {
                    if cfg!(windows) {
                        ";".into()
                    } else {
                        ":".into()
                    }
                }),
            open_delim: args
                .open_delim
                .or(file_config.as_ref().and_then(|c| c.open_delim.clone()))
                .unwrap_or_else(|| "<[".into()),
            close_delim: args
                .close_delim
                .or(file_config.as_ref().and_then(|c| c.close_delim.clone()))
                .unwrap_or_else(|| "]>".into()),
            chunk_end: args
                .chunk_end
                .or(file_config.as_ref().and_then(|c| c.chunk_end.clone()))
                .unwrap_or_else(|| "$$".into()),
            comment_markers: args
                .comment_markers
                .or(file_config.as_ref().and_then(|c| c.comment_markers.clone()))
                .unwrap_or_else(|| "#,//".into()),
            chunks: args.chunks,
            pydef: args.pydef || file_config.as_ref().and_then(|c| c.pydef).unwrap_or(false),
            dump_ast: args.dump_ast
                || file_config
                    .as_ref()
                    .and_then(|c| c.dump_ast)
                    .unwrap_or(false),
        })
    }
}
