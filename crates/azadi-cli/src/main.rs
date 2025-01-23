use azadi_macros::evaluator::{evaluator::EvalConfig, EvalError, Evaluator};
use azadi_macros::macro_api;
use azadi_noweb::noweb::Clip;
use azadi_noweb::safe_writer::SafeFileWriter;
use azadi_noweb::AzadiError;
use clap::Parser;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Macro processing error: {0}")]
    MacroError(#[from] EvalError),

    #[error("Noweb processing error: {0}")]
    NowebError(#[from] AzadiError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("String conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Input error: {0}")]
    InputError(String),
}

fn process_noweb_phase(
    input: &PathBuf,
    clip: &mut Clip,
    chunks: &Option<String>,
) -> Result<(), PipelineError> {
    let content = read_input(input)?;
    clip.read(&content, input.to_string_lossy().as_ref());

    if let Some(chunk_list) = chunks {
        // Extract specific chunks to stdout
        for chunk in chunk_list.split(',') {
            let expanded = clip.expand(chunk.trim(), "")?;
            for line in expanded {
                println!("{}", line);
            }
        }
    } else {
        // Normal file processing - writes to configured output paths
        clip.write_files()?;
    }
    Ok(())
}

/// Returns the default path separator based on the platform
fn default_pathsep() -> String {
    if cfg!(windows) {
        ";".to_string()
    } else {
        ":".to_string()
    }
}

/// Azadi pipeline combining macro processing and literate programming
#[derive(Parser, Debug)]
#[command(name = "azadi", about = "Process files through macro expansion and literate programming", long_about = None)]
struct Args {
    /// Input files to process (use "-" for stdin)
    #[arg(
        required = true,
        help = "One or more input files to process. Use - for stdin"
    )]
    files: Vec<PathBuf>,

    /// Base directory for input files (ignored for stdin)
    #[arg(
        long = "input-dir",
        default_value = ".",
        help = "Base directory for resolving input files (ignored when reading from stdin)"
    )]
    input_dir: PathBuf,

    /// Output directory for generated files
    #[arg(long, default_value = "gen", help = "Directory for output files")]
    output_dir: PathBuf,

    /// Special character used to mark macro invocations
    #[arg(long, default_value = "%", help = "Special character for macro syntax")]
    special: char,

    /// Working directory for temporary and backup files
    #[arg(
        long,
        default_value = "_azadi_work",
        help = "Directory for temporary and backup files"
    )]
    work_dir: PathBuf,

    /// Keep intermediate files from macro processing
    #[arg(long, help = "Save intermediate macro output files", action = clap::ArgAction::SetTrue)]
    save_macro: bool,

    /// Only perform macro processing
    #[arg(long, help = "Stop after macro processing", action = clap::ArgAction::SetTrue)]
    macro_only: bool,

    /// Skip macro processing and only run noweb
    #[arg(long, help = "Skip macro processing", action = clap::ArgAction::SetTrue)]
    noweb_only: bool,

    /// List of include paths for macro processing, separated by pathsep
    #[arg(
        long,
        default_value = ".",
        help = "Pathsep-separated list of include paths"
    )]
    include: String,

    /// Path separator for include paths and other lists
    #[arg(long = "pathsep", default_value_t = default_pathsep(),
          help = "Path separator character (; on Windows, : on Unix-like systems)")]
    pathsep: String,

    /// Opening delimiter for chunk definitions
    #[arg(
        long,
        default_value = "<[",
        help = "Opening delimiter for chunk definitions"
    )]
    open_delim: String,

    /// Closing delimiter for chunk definitions
    #[arg(
        long,
        default_value = "]>",
        help = "Closing delimiter for chunk definitions"
    )]
    close_delim: String,

    /// End marker for chunk definitions
    #[arg(
        long,
        default_value = "$$",
        help = "Marker that ends a chunk definition"
    )]
    chunk_end: String,

    /// Comment markers used in source files
    #[arg(
        long,
        default_value = "#,//",
        help = "Comma-separated list of comment markers"
    )]
    comment_markers: String,

    /// Specific chunks to extract
    #[arg(long, help = "Comma-separated list of chunks to extract")]
    chunks: Option<String>,

    /// Enable Python macro definitions
    #[arg(long = "pydef", help = "Enable Python macro definitions", action = clap::ArgAction::SetTrue)]
    pydef: bool,

    /// Parse input and emit AST in JSONL format
    #[arg(long = "dump_ast", help = "Parse input and emit AST in JSONL format", action = clap::ArgAction::SetTrue)]
    dump_ast: bool,
}

fn setup_directories(args: &Args) -> Result<PathBuf, PipelineError> {
    let macro_dir = args.work_dir.join("macro_out");
    std::fs::create_dir_all(&macro_dir)?;
    std::fs::create_dir_all(&args.output_dir)?;
    Ok(macro_dir)
}

fn read_input(path: &PathBuf) -> io::Result<String> {
    if is_stdio_path(path) {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        std::fs::read_to_string(path)
    }
}

fn write_output(path: &PathBuf, content: &[u8]) -> io::Result<()> {
    if is_stdio_path(path) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(content)?;
        handle.flush()
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }
}

fn process_macro_phase(
    input: &PathBuf,
    output: &PathBuf,
    config: &EvalConfig,
) -> Result<(), PipelineError> {
    let content = read_input(input)?;

    let result =
        macro_api::process_string(&content, Some(input), &mut Evaluator::new(config.clone()))
            .map_err(PipelineError::MacroError)?;

    write_output(output, &result)?;
    Ok(())
}

fn resolve_input_paths(args: &Args) -> Result<Vec<PathBuf>, PipelineError> {
    let mut final_inputs = Vec::new();

    for inp in &args.files {
        let full = args.input_dir.join(inp);

        // Try to get canonical path for better error messages
        let canon = full.canonicalize().unwrap_or_else(|_| full.clone());

        if !full.exists() {
            return Err(PipelineError::InputError(format!(
                "Input file does not exist: {:?}",
                canon
            )));
        }
        eprintln!("Processing input file: {:?}", canon);
        final_inputs.push(full);
    }

    Ok(final_inputs)
}

fn is_stdio_path(path: &PathBuf) -> bool {
    path.to_str() == Some("-")
}

fn get_macro_output_path(input: &PathBuf, macro_dir: &PathBuf) -> PathBuf {
    if is_stdio_path(input) {
        PathBuf::from("-")
    } else {
        macro_dir
            .join(input.file_name().unwrap())
            .with_extension("txt")
    }
}

fn run_pipeline(args: Args) -> Result<(), PipelineError> {
    let input_files = resolve_input_paths(&args)?;

    // If --dump-ast specified, emit AST and exit
    if args.dump_ast {
        azadi_macros::ast::serialization::dump_macro_ast(args.special, &input_files)
            .map_err(PipelineError::MacroError)?;
        return Ok(());
    }

    // Configure macro processing
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

    // Configure noweb processing
    let safe_writer = SafeFileWriter::new(&args.output_dir);
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
        let dir = setup_directories(&args)?;
        Some(dir)
    } else {
        None
    };

    // Process each input file
    for input in input_files {
        if !args.noweb_only {
            let macro_out = get_macro_output_path(&input, macro_dir.as_ref().unwrap());
            process_macro_phase(&input, &macro_out, &macro_config)?;

            // If macro_only and using stdout, we're done
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
            process_noweb_phase(&noweb_input, &mut clip, &args.chunks)?;
        }
    }

    // Only cleanup if we created a macro_dir
    if !args.save_macro && !args.macro_only && macro_dir.is_some() {
        std::fs::remove_dir_all(macro_dir.unwrap())?;
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    match run_pipeline(args) {
        Ok(()) => {}
        Err(PipelineError::MacroError(EvalError::Terminate(_))) => {
            eprintln!("%here macro executed successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Additional error context
            match e {
                PipelineError::MacroError(e) => eprintln!("During macro processing: {}", e),
                PipelineError::NowebError(e) => eprintln!("During noweb processing: {}", e),
                PipelineError::IoError(e) => eprintln!("IO operation failed: {}", e),
                PipelineError::Utf8Error(e) => eprintln!("String encoding error: {}", e),
                PipelineError::InputError(e) => eprintln!("Input file error: {}", e),
            }
            std::process::exit(1);
        }
    }
}
