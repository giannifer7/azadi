//macro_cli.rs

use azadi_macros::evaluator::evaluator::EvalConfig;
use clap::Parser;
use std::path::PathBuf;

use azadi_macros::evaluator::EvalError;
use azadi_macros::macro_api::process_files_from_config;

/// Returns the default path separator based on the platform
fn default_pathsep() -> String {
    if cfg!(windows) {
        ";".to_string()
    } else {
        ":".to_string()
    }
}

#[derive(Parser, Debug)]
#[command(name = "azadi-macro-cli", about = "Azadi macros translator (Rust)")]
struct Args {
    /// Output directory
    #[arg(long = "out-dir", default_value = ".")]
    out_dir: PathBuf,

    /// Special character for macros
    #[arg(long = "special", default_value = "%")]
    special: char,

    /// Working dir for backups
    #[arg(long = "work-dir", default_value = "_azadi_work")]
    work_dir: PathBuf,

    /// Colon-separated list of include paths
    #[arg(long = "include", default_value = ".")]
    include: String,

    /// Path separator (usually ':' on Unix, ';' on Windows)
    #[arg(long = "pathsep", default_value_t = default_pathsep())]
    pathsep: String,

    /// If set, python macros are considered
    #[arg(long = "pydef", default_value = "false")]
    pydef: bool,

    /// Base directory for input files
    #[arg(long = "input-dir", default_value = ".")]
    input_dir: PathBuf,

    /// The input files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    match run(args) {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<(), EvalError> {
    let include_paths: Vec<PathBuf> = args
        .include
        .split(&args.pathsep)
        .map(PathBuf::from)
        .collect();

    let config = EvalConfig {
        special_char: args.special,
        pydef: args.pydef,
        include_paths,
        backup_dir: args.work_dir,
    };

    let mut final_inputs = Vec::new();
    for inp in &args.inputs {
        let full = args.input_dir.join(inp);
        if !full.exists() {
            return Err(EvalError::Runtime(format!(
                "Input file does not exist: {full:?}"
            )));
        }
        final_inputs.push(full);
    }

    process_files_from_config(&final_inputs, &args.out_dir, config)
}
