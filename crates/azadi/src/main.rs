// crates/azadi/src/main.rs
//
// Combined macro-expander + literate-programming extractor.
// Runs azadi-macros then azadi-noweb in-process, no subprocess spawning.

use azadi_macros::{
    evaluator::{EvalConfig, EvalError, Evaluator},
    macro_api::process_string,
};
use azadi_noweb::{AzadiError, Clip, SafeFileWriter, SafeWriterConfig};
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

fn default_pathsep() -> String {
    if cfg!(windows) { ";".to_string() } else { ":".to_string() }
}

#[derive(Parser, Debug)]
#[command(
    name = "azadi",
    about = "Macro expander + literate-programming chunk extractor in one pass"
)]
struct Args {
    /// Input files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    // ── azadi-macros options ──────────────────────────────────────────────────

    /// Special character for macros
    #[arg(long, default_value = "%")]
    special: char,

    /// Include paths for %include/%import (colon-separated on Unix)
    #[arg(long, default_value = ".")]
    include: String,

    /// Work directory (macro backups + noweb private files)
    #[arg(long, default_value = "_azadi_work")]
    work_dir: PathBuf,

    // ── azadi-noweb options ───────────────────────────────────────────────────

    /// Base directory for generated output files
    #[arg(long = "gen", default_value = "gen")]
    gen_dir: PathBuf,

    /// Chunk open delimiter
    #[arg(long, default_value = "<<")]
    open_delim: String,

    /// Chunk close delimiter
    #[arg(long, default_value = ">>")]
    close_delim: String,

    /// Chunk end marker
    #[arg(long, default_value = "@")]
    chunk_end: String,

    /// Comment markers recognised before chunk delimiters (comma-separated)
    #[arg(long, default_value = "#,//")]
    comment_markers: String,

    /// Formatter command per output file extension, e.g. --formatter rs=rustfmt
    #[arg(long, value_name = "EXT=CMD")]
    formatter: Vec<String>,
}

#[derive(Debug)]
enum Error {
    Macro(EvalError),
    Noweb(AzadiError),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Macro(e) => write!(f, "{e}"),
            Error::Noweb(e) => write!(f, "{e}"),
            Error::Io(e) => write!(f, "{e}"),
        }
    }
}

impl From<EvalError> for Error {
    fn from(e: EvalError) -> Self { Error::Macro(e) }
}
impl From<AzadiError> for Error {
    fn from(e: AzadiError) -> Self { Error::Noweb(e) }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::Io(e) }
}

fn run(args: Args) -> Result<(), Error> {
    let pathsep = default_pathsep();
    let include_paths: Vec<PathBuf> = args.include.split(&pathsep).map(PathBuf::from).collect();

    std::fs::create_dir_all(&args.work_dir)
        .map_err(Error::Io)?;

    let eval_config = EvalConfig {
        special_char: args.special,
        include_paths,
        backup_dir: args.work_dir.clone(),
    };
    let mut evaluator = Evaluator::new(eval_config);

    let comment_markers: Vec<String> = args
        .comment_markers
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let formatters: HashMap<String, String> = args
        .formatter
        .iter()
        .filter_map(|s| s.split_once('=').map(|(e, c)| (e.to_string(), c.to_string())))
        .collect();

    let safe_writer = SafeFileWriter::with_config(
        &args.gen_dir,
        &args.work_dir,
        SafeWriterConfig {
            formatters,
            ..SafeWriterConfig::default()
        },
    );
    let mut clip = Clip::new(
        safe_writer,
        &args.open_delim,
        &args.close_delim,
        &args.chunk_end,
        &comment_markers,
    );

    // Phase 1: macro-expand each input file, feed result to noweb.
    for input_path in &args.inputs {
        let content = std::fs::read_to_string(input_path)?;
        let expanded = process_string(&content, Some(input_path), &mut evaluator)?;
        let expanded_str = String::from_utf8_lossy(&expanded);
        clip.read(&expanded_str, &input_path.to_string_lossy());
    }

    // Phase 2: write all @file chunks.
    clip.write_files()?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
