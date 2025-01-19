// src/main.rs

use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use azadi_noweb::noweb::Clip;
use azadi_noweb::safe_writer::SafeFileWriter;
use azadi_noweb::AzadiError;

#[derive(Parser, Debug)]
#[command(name = "azadi-noweb", version, about = "Expand chunks like noweb")]
struct Args {
    #[arg(long, default_value = "<<")]
    open_delim: String,

    #[arg(long, default_value = ">>")]
    close_delim: String,

    #[arg(long, default_value = "@")]
    chunk_end: String,

    #[arg(long, default_value = "#")]
    comment_markers: String,

    #[arg(long, default_value = "_azadi_work")]
    _priv_dir: PathBuf,

    #[arg(long, default_value = "gen")]
    gen: PathBuf,

    #[arg(long)]
    chunks: Option<String>,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn write_chunks(
    clip: &mut Clip,
    chunk_list: &[&str],
    out: &mut dyn Write,
) -> Result<(), AzadiError> {
    for chunk_name in chunk_list {
        let lines = clip.expand(chunk_name, "")?;
        for line in lines {
            out.write_all(line.as_bytes())?;
        }
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn run(args: Args) -> Result<(), AzadiError> {
    let cmarkers: Vec<String> = args
        .comment_markers
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let safe_writer = SafeFileWriter::new(&args.gen);
    let mut clip = Clip::new(
        safe_writer,
        &args.open_delim,
        &args.close_delim,
        &args.chunk_end,
        &cmarkers,
    );

    // read all input files
    clip.read_files(&args.files)?;

    // expand or write
    if let Some(chunks) = args.chunks {
        let chunk_names: Vec<&str> = chunks.split(',').collect();
        if let Some(outpath) = args.output {
            let mut f = File::create(outpath)?;
            write_chunks(&mut clip, &chunk_names, &mut f)?;
        } else {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            write_chunks(&mut clip, &chunk_names, &mut handle)?;
        }
    } else {
        clip.write_files()?;
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
