// crates/azadi-macros/src/bin/macro_cli.rs

use azadi_macros::evaluator::{EvalConfig, EvalError, PythonConfig};
use clap::Parser;
use std::path::PathBuf;

/// Returns the default path separator based on the platform
fn default_pathsep() -> String {
    if cfg!(windows) {
        ";".to_string()
    } else {
        ":".to_string()
    }
}

#[derive(Parser, Debug)]
#[command(name = "azadi-macros", about = "Azadi macros translator (Rust)")]
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

    /// List of include paths separated by the path separator
    #[arg(long = "include", default_value = ".")]
    include: String,

    /// Path separator (usually ':' on Unix, ';' on Windows)
    #[arg(long = "pathsep", default_value_t = default_pathsep())]
    pathsep: String,

    /// Path to Python executable or venv directory
    #[arg(long = "python-path")]
    python_path: Option<PathBuf>,

    /// If set, python macros are considered
    #[arg(long = "pydef", default_value_t = false)]
    pydef: bool,

    /// Base directory for input files
    #[arg(long = "input-dir", default_value = ".")]
    input_dir: PathBuf,

    /// The input files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
}

fn run(args: Args) -> Result<(), EvalError> {
    let include_paths: Vec<PathBuf> = args
        .include
        .split(&args.pathsep)
        .map(PathBuf::from)
        .collect();

    let (venv_path, python_path) = if let Some(path) = args.python_path {
        if path.is_dir() {
            (Some(path), None)
        } else {
            (None, Some(path))
        }
    } else {
        (None, None)
    };

    let python_config = PythonConfig {
        enabled: true,
        venv_path,
        python_path,
    };

    let config = EvalConfig {
        special_char: args.special,
        pydef: args.pydef,
        include_paths,
        backup_dir: args.work_dir.clone(),
        python: python_config,
    };

    // Ensure work directory exists
    if !args.work_dir.exists() {
        std::fs::create_dir_all(&args.work_dir)
            .map_err(|e| EvalError::Runtime(format!("Failed to create work directory: {}", e)))?;
    }

    let mut final_inputs = Vec::new();
    for inp in &args.inputs {
        let full = args.input_dir.join(inp);
        // Try to get canonical path for better error messages
        let canon = full.canonicalize().unwrap_or_else(|_| full.clone());
        if !full.exists() {
            return Err(EvalError::Runtime(format!(
                "Input file does not exist: {:?}",
                canon
            )));
        }
        final_inputs.push(full);
    }

    let result =
        azadi_macros::macro_api::process_files_from_config(&final_inputs, &args.out_dir, config);

    result
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
