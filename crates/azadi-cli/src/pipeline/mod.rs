// crates/azadi-cli/src/pipeline/mod.rs
use crate::{try_macro, try_mkdir, try_noweb, try_read, try_write};
use azadi_macros::evaluator::{evaluator::EvalConfig, EvalError, Evaluator};
use azadi_macros::macro_api;
use azadi_noweb::noweb::Clip;
use azadi_noweb::safe_writer::SafeFileWriter;
use azadi_noweb::AzadiError;
use clap::Parser;
use std::backtrace::Backtrace;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use thiserror::Error;

#[macro_use]
mod error_macros;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Failed to read file {path:?}: {source} (at {backtrace})")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[error("Failed to write file {path:?}: {source} (at {backtrace})")]
    WriteError {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[error("Failed to create directory {path:?}: {source} (at {backtrace})")]
    CreateDirError {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[error("Input file not found: {path:?} (at {backtrace})")]
    InputNotFound { path: PathBuf, backtrace: Backtrace },

    #[error("Macro processing error: {source} (at {backtrace})")]
    MacroError {
        source: EvalError,
        backtrace: Backtrace,
    },

    #[error("Noweb processing error: {source} (at {backtrace})")]
    NowebError {
        source: AzadiError,
        backtrace: Backtrace,
    },

    #[error("String conversion error: {source} (at {backtrace})")]
    Utf8Error {
        source: std::string::FromUtf8Error,
        backtrace: Backtrace,
    },
}

impl From<std::io::Error> for PipelineError {
    fn from(err: std::io::Error) -> Self {
        PipelineError::ReadError {
            path: PathBuf::from("<unknown>"),
            source: err,
            backtrace: Backtrace::capture(),
        }
    }
}

pub fn read_input(path: &PathBuf) -> Result<String, PipelineError> {
    if is_stdio_path(path) {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        try_read!(path)
    }
}

pub fn write_output(path: &PathBuf, content: &[u8]) -> Result<(), PipelineError> {
    if is_stdio_path(path) {
        io::stdout().write_all(content)?;
        io::stdout().flush()?;
        Ok(())
    } else {
        try_write!(path, content)
    }
}

pub fn setup_directories(args: &Args) -> Result<PathBuf, PipelineError> {
    let work_dir = args
        .work_dir
        .canonicalize()
        .unwrap_or_else(|_| args.work_dir.clone());
    let macro_dir = work_dir.join("macro_out");

    try_mkdir!(&work_dir)?;
    try_mkdir!(&macro_dir)?;
    try_mkdir!(&args.output_dir)?;

    Ok(macro_dir)
}

pub fn process_macro_phase(
    input: &PathBuf,
    output: &PathBuf,
    config: &EvalConfig,
) -> Result<(), PipelineError> {
    let content = read_input(input)?;
    let result = try_macro!(macro_api::process_string(
        &content,
        Some(input),
        &mut Evaluator::new(config.clone())
    ))?;
    write_output(output, &result)
}

pub fn process_noweb_phase(
    input: &PathBuf,
    clip: &mut Clip,
    chunks: &Option<String>,
) -> Result<(), PipelineError> {
    let content = read_input(input)?;
    clip.read(&content, input.to_string_lossy().as_ref());

    if let Some(chunk_list) = chunks {
        for chunk in chunk_list.split(',') {
            let expanded = try_noweb!(clip.expand(chunk.trim(), ""))?;
            for line in expanded {
                println!("{}", line);
            }
        }
    } else {
        try_noweb!(clip.write_files())?;
    }
    Ok(())
}

pub fn resolve_input_paths(args: &Args) -> Result<Vec<PathBuf>, PipelineError> {
    let mut final_inputs = Vec::new();

    for inp in &args.files {
        if is_stdio_path(inp) {
            final_inputs.push(inp.clone());
            continue;
        }

        let path = if inp.is_absolute() {
            inp.clone()
        } else {
            args.input_dir.join(inp)
        };

        if !path.exists() {
            return Err(PipelineError::InputNotFound {
                path: path.clone(),
                backtrace: Backtrace::capture(),
            });
        }

        final_inputs.push(path);
    }

    Ok(final_inputs)
}

pub fn is_stdio_path(path: &PathBuf) -> bool {
    path.to_str() == Some("-")
}

pub fn get_macro_output_path(input: &PathBuf, macro_dir: &PathBuf) -> PathBuf {
    if is_stdio_path(input) {
        PathBuf::from("-")
    } else {
        macro_dir
            .join(input.file_name().unwrap())
            .with_extension("txt")
    }
}

/// Returns the default path separator based on the platform
fn default_pathsep() -> String {
    if cfg!(windows) {
        ";".to_string()
    } else {
        ":".to_string()
    }
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
        default_value = ".",
        help = "Base directory for resolving input files (ignored when reading from stdin)"
    )]
    pub input_dir: PathBuf,

    /// Output directory for generated files
    #[arg(long, default_value = "gen", help = "Directory for output files")]
    pub output_dir: PathBuf,

    /// Special character used to mark macro invocations
    #[arg(long, default_value = "%", help = "Special character for macro syntax")]
    pub special: char,

    /// Working directory for temporary and backup files
    #[arg(
        long,
        default_value = "_azadi_work",
        help = "Directory for temporary and backup files"
    )]
    pub work_dir: PathBuf,

    /// Keep intermediate files from macro processing
    #[arg(long, help = "Save intermediate macro output files", action = clap::ArgAction::SetTrue)]
    pub save_macro: bool,

    /// Only perform macro processing
    #[arg(long, help = "Stop after macro processing", action = clap::ArgAction::SetTrue)]
    pub macro_only: bool,

    /// Skip macro processing and only run noweb
    #[arg(long, help = "Skip macro processing", action = clap::ArgAction::SetTrue)]
    pub noweb_only: bool,

    /// Colon-separated list of include paths for macro processing
    #[arg(
        long,
        default_value = ".",
        help = "Pathsep-separated list of include paths"
    )]
    pub include: String,

    /// Path separator for include paths
    #[arg(long = "pathsep", default_value_t = default_pathsep(),
          help = "Path separator character (; on Windows, : on Unix-like systems)")]
    pub pathsep: String,

    /// Opening delimiter for chunk definitions
    #[arg(
        long,
        default_value = "<[",
        help = "Opening delimiter for chunk definitions"
    )]
    pub open_delim: String,

    /// Closing delimiter for chunk definitions
    #[arg(
        long,
        default_value = "]>",
        help = "Closing delimiter for chunk definitions"
    )]
    pub close_delim: String,

    /// End marker for chunk definitions
    #[arg(
        long,
        default_value = "$$",
        help = "Marker that ends a chunk definition"
    )]
    pub chunk_end: String,

    /// Comment markers used in source files
    #[arg(
        long,
        default_value = "#,//",
        help = "Comma-separated list of comment markers"
    )]
    pub comment_markers: String,

    /// Specific chunks to extract
    #[arg(long, help = "Comma-separated list of chunks to extract")]
    pub chunks: Option<String>,

    /// Enable Python macro definitions
    #[arg(long = "pydef", help = "Enable Python macro definitions", action = clap::ArgAction::SetTrue)]
    pub pydef: bool,

    /// Parse input and emit AST in JSONL format
    #[arg(long = "dump_ast", help = "Parse input and emit AST in JSONL format", action = clap::ArgAction::SetTrue)]
    pub dump_ast: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            files: vec![],
            input_dir: ".".into(),
            output_dir: "gen".into(),
            special: '%',
            work_dir: "_azadi_work".into(),
            save_macro: false,
            macro_only: false,
            noweb_only: false,
            include: ".".into(),
            pathsep: default_pathsep(),
            open_delim: "<[".into(),
            close_delim: "]>".into(),
            chunk_end: "$$".into(),
            comment_markers: "#,//".into(),
            chunks: None,
            pydef: false,
            dump_ast: false,
        }
    }
}

pub fn run_pipeline(args: Args) -> Result<(), PipelineError> {
    let input_files = resolve_input_paths(&args)?;

    if args.dump_ast {
        try_macro!(azadi_macros::ast::serialization::dump_macro_ast(
            args.special,
            &input_files
        ))?;
        return Ok(());
    }

    let macro_config = EvalConfig {
        special_char: args.special,
        pydef: args.pydef,
        include_paths: args
            .include
            .split(&args.pathsep)
            .map(PathBuf::from)
            .collect(),
        backup_dir: args.work_dir.clone(),
    };

    let safe_writer = SafeFileWriter::new(&args.output_dir, &args.work_dir);
    let comment_markers: Vec<String> = args
        .comment_markers
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect();

    let mut clip = Clip::new(
        safe_writer,
        &args.open_delim,
        &args.close_delim,
        &args.chunk_end,
        &comment_markers,
    );

    let macro_dir = if !args.noweb_only && !is_stdio_path(&input_files[0]) {
        Some(setup_directories(&args)?)
    } else {
        None
    };

    for input in input_files {
        if !args.noweb_only {
            let macro_out = get_macro_output_path(&input, macro_dir.as_ref().unwrap());
            if let Err(e) = process_macro_phase(&input, &macro_out, &macro_config) {
                eprintln!("Macro phase failed: {:?}", e);
                return Err(e);
            }

            if args.macro_only && is_stdio_path(&macro_out) {
                continue;
            }
        }

        if !args.macro_only {
            let noweb_input = if args.noweb_only {
                input.clone()
            } else {
                get_macro_output_path(&input, macro_dir.as_ref().unwrap())
            };
            if let Err(e) = process_noweb_phase(&noweb_input, &mut clip, &args.chunks) {
                eprintln!("Noweb phase failed: {:?}", e);
                return Err(e);
            }
        }
    }

    if !args.save_macro && !args.macro_only && macro_dir.is_some() {
        std::fs::remove_dir_all(macro_dir.unwrap())?;
    }

    Ok(())
}
