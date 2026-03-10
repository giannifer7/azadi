// crates/azadi/src/main.rs
//
// Combined macro-expander + literate-programming extractor.
// Runs azadi-macros then azadi-noweb in-process, no subprocess spawning.

use azadi_macros::{
    evaluator::{EvalConfig, EvalError, Evaluator},
    macro_api::process_string,
};
use azadi_noweb::{AzadiError, Clip, SafeFileWriter, SafeWriterConfig};
use clap::{ArgGroup, Parser};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

fn default_pathsep() -> String {
    if cfg!(windows) { ";".to_string() } else { ":".to_string() }
}

#[derive(Parser, Debug)]
#[command(
    name = "azadi",
    about = "Macro expander + literate-programming chunk extractor in one pass",
    group(ArgGroup::new("source").required(true).args(["inputs", "directory"]))
)]
struct Args {
    /// Input files (mutually exclusive with --directory)
    #[arg(required = false)]
    inputs: Vec<PathBuf>,

    // ── azadi-macros options ──────────────────────────────────────────────────

    /// Base directory prepended to every input path
    #[arg(long, default_value = ".")]
    input_dir: PathBuf,

    /// Special character for macros
    #[arg(long, default_value = "%")]
    special: char,

    /// Include paths for %include/%import (colon-separated on Unix)
    #[arg(long, default_value = ".")]
    include: String,

    /// Work directory (macro backups + noweb private files)
    #[arg(long, default_value = "_azadi_work")]
    work_dir: PathBuf,

    // ── debugging ─────────────────────────────────────────────────────────────

    /// Print macro-expanded text to stderr before noweb processing
    #[arg(long)]
    dump_expanded: bool,

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

    // ── batch/directory mode ──────────────────────────────────────────────────

    /// Discover and process all .adoc driver files under this directory.
    /// A driver is any .adoc not referenced by a %include() in another .adoc.
    /// Mutually exclusive with positional input files.
    #[arg(long, conflicts_with = "inputs")]
    directory: Option<PathBuf>,

    // ── build-system integration ──────────────────────────────────────────────

    /// Write a Makefile depfile listing every source file read.
    /// In --directory mode the depfile lists ALL .adoc files found so that
    /// adding a new file triggers a rebuild.
    #[arg(long)]
    depfile: Option<PathBuf>,

    /// Touch this file on success (build-system stamp).
    #[arg(long)]
    stamp: Option<PathBuf>,
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

/// Recursively collect all files matching `ext` under `dir`.
fn find_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            find_files(&path, ext, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
    Ok(())
}

/// Scan a file for `{special}include(path)` patterns and return resolved paths.
fn collect_includes(file: &Path, special: char, include_root: &Path) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(file) else { return vec![] };
    // Both %include and %import pull in another file and mark it as a fragment.
    let prefixes = [
        format!("{}include(", special),
        format!("{}import(", special),
    ];
    let mut result = Vec::new();
    for prefix in &prefixes {
        let mut rest = text.as_str();
        while let Some(pos) = rest.find(prefix.as_str()) {
            rest = &rest[pos + prefix.len()..];
            if let Some(end) = rest.find(')') {
                let path_str = rest[..end].trim();
                result.push(include_root.join(path_str));
                rest = &rest[end + 1..];
            }
        }
    }
    result
}

/// Escape a path for use in a Makefile depfile (spaces → `\ `).
fn depfile_escape(p: &Path) -> String {
    p.to_string_lossy().replace(' ', "\\ ")
}

/// Write a Makefile depfile.  `target` is the stamp; `deps` are all inputs.
fn write_depfile(path: &Path, target: &Path, deps: &[PathBuf]) -> std::io::Result<()> {
    use std::fmt::Write as FmtWrite;
    let mut out = String::new();
    write!(out, "{}:", depfile_escape(target)).unwrap();
    for dep in deps {
        write!(out, " {}", depfile_escape(dep)).unwrap();
    }
    out.push('\n');
    std::fs::write(path, out)
}

fn run(args: Args) -> Result<(), Error> {
    let pathsep = default_pathsep();
    let include_paths: Vec<PathBuf> = args.include.split(&pathsep).map(PathBuf::from).collect();

    std::fs::create_dir_all(&args.work_dir).map_err(Error::Io)?;

    let eval_config = EvalConfig {
        special_char: args.special,
        include_paths: include_paths.clone(),
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
        SafeWriterConfig { formatters, ..SafeWriterConfig::default() },
    );
    let mut clip = Clip::new(
        safe_writer,
        &args.open_delim,
        &args.close_delim,
        &args.chunk_end,
        &comment_markers,
    );

    // Determine the set of driver files to process and all .adoc files for the depfile.
    let (drivers, all_adoc): (Vec<PathBuf>, Vec<PathBuf>) = if let Some(ref dir) = args.directory {
        let mut all = Vec::new();
        find_files(dir, "adoc", &mut all).map_err(Error::Io)?;
        all.sort();

        // include_root for resolving %include paths (first include_path or dir itself)
        let include_root = include_paths.first().map(PathBuf::as_path).unwrap_or(dir.as_path());

        // Collect all paths referenced by %include(...) across the tree.
        let mut included: HashSet<PathBuf> = HashSet::new();
        for adoc in &all {
            for p in collect_includes(adoc, args.special, include_root) {
                included.insert(p.canonicalize().unwrap_or(p));
            }
        }

        let drivers = all
            .iter()
            .filter(|f| {
                let canon = f.canonicalize().unwrap_or_else(|_| f.to_path_buf());
                !included.contains(&canon)
            })
            .cloned()
            .collect();

        (drivers, all)
    } else {
        let drivers = args
            .inputs
            .iter()
            .map(|p| args.input_dir.join(p))
            .collect::<Vec<_>>();
        (drivers.clone(), drivers)
    };

    // Phase 1: macro-expand each driver, feed result to noweb.
    for full_path in &drivers {
        let content = std::fs::read_to_string(full_path)?;
        let expanded = process_string(&content, Some(full_path), &mut evaluator)?;
        let expanded_str = String::from_utf8_lossy(&expanded);
        if args.dump_expanded {
            eprintln!("=== expanded: {} ===", full_path.display());
            eprintln!("{}", expanded_str);
            eprintln!("=== end: {} ===", full_path.display());
        }
        clip.read(&expanded_str, &full_path.to_string_lossy());
    }

    // Phase 2: write all @file chunks.
    clip.write_files()?;

    // Write depfile if requested.
    if let Some(ref depfile_path) = args.depfile {
        // In directory mode: depend on all .adoc so adding a new file triggers rebuild.
        // In file mode: depend only on files actually read by the evaluator.
        let deps: Vec<PathBuf> = if args.directory.is_some() {
            all_adoc
        } else {
            evaluator.source_files().to_vec()
        };
        let stamp_path = args.stamp.clone().unwrap_or_else(|| depfile_path.clone());
        write_depfile(depfile_path, &stamp_path, &deps).map_err(Error::Io)?;
    }

    // Touch stamp file if requested.
    if let Some(ref stamp_path) = args.stamp {
        std::fs::write(stamp_path, b"").map_err(Error::Io)?;
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
