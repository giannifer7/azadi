// crates/azadi-macros/src/bin/macro_cli.rs

use azadi_macros::evaluator::{EvalConfig, EvalError, PyBackend, PythonConfig, SecurityLevel};
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

    /// Python backend to use
    #[arg(long = "python-backend", value_enum, default_value_t = PyBackend::None)]
    python_backend: PyBackend,

    /// Python security level
    #[arg(long = "python-security", value_enum, default_value_t = SecurityLevel::Basic)]
    python_security: SecurityLevel,

    /// Path to Python virtual environment
    #[arg(long = "venv-path")]
    venv_path: Option<PathBuf>,

    /// Path to Python executable
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
    eprintln!("Starting macro-cli with arguments: {:?}", args);

    let include_paths: Vec<PathBuf> = args
        .include
        .split(&args.pathsep)
        .map(PathBuf::from)
        .collect();

    eprintln!("Include paths: {:?}", include_paths);
    eprintln!("Special character: {}", args.special);
    eprintln!("Python backend: {:?}", args.python_backend);
    eprintln!("Python security level: {:?}", args.python_security);

    let python_enabled = match args.python_backend {
        PyBackend::None => false,
        _ => true,
    };

    let python_config = PythonConfig {
        enabled: python_enabled,
        venv_path: args.venv_path,
        python_path: args.python_path,
        security_level: args.python_security,
    };

    let config = EvalConfig {
        special_char: args.special,
        pydef: args.pydef,
        include_paths,
        backup_dir: args.work_dir.clone(),
        python: python_config,
    };

    // Ensure output directory exists
    if !args.out_dir.exists() {
        eprintln!("Creating output directory: {:?}", args.out_dir);
        std::fs::create_dir_all(&args.out_dir)
            .map_err(|e| EvalError::Runtime(format!("Failed to create output directory: {}", e)))?;
    }

    // Ensure work directory exists
    if !args.work_dir.exists() {
        eprintln!("Creating work directory: {:?}", args.work_dir);
        std::fs::create_dir_all(&args.work_dir)
            .map_err(|e| EvalError::Runtime(format!("Failed to create work directory: {}", e)))?;
    }

    let mut final_inputs = Vec::new();
    eprintln!(
        "Processing input files from base directory: {:?}",
        args.input_dir
    );
    for inp in &args.inputs {
        let full = args.input_dir.join(inp);
        eprintln!("Checking input file: {:?}", full);

        // Try to get canonical path for better error messages
        let canon = full.canonicalize().unwrap_or_else(|_| full.clone());

        if !full.exists() {
            return Err(EvalError::Runtime(format!(
                "Input file does not exist: {:?}",
                canon
            )));
        }
        eprintln!("Input file exists: {:?}", canon);
        final_inputs.push(full);
    }

    eprintln!("Starting file processing with:");
    eprintln!("  Output directory: {:?}", args.out_dir);
    eprintln!("  Work directory: {:?}", args.work_dir);
    eprintln!("  Final inputs: {:?}", final_inputs);

    let result =
        azadi_macros::macro_api::process_files_from_config(&final_inputs, &args.out_dir, config);

    if let Err(ref e) = result {
        eprintln!("Processing failed: {:?}", e);
    } else {
        eprintln!("Processing completed successfully");
    }

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
